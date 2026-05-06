import AppKit
import Foundation

@main
final class ArcadiaDevelopmentLauncher: NSObject, NSApplicationDelegate, NSMenuDelegate {
    private let statusItem = NSStatusBar.system.statusItem(withLength: NSStatusItem.variableLength)
    private let repositoryRoot = URL(fileURLWithPath: NSHomeDirectory()).appendingPathComponent("Arcadia")
    private let processQueue = DispatchQueue(label: "arcadia.development-launcher.process")

    private var startItem: NSMenuItem!
    private var restartItem: NSMenuItem!
    private var stopItem: NSMenuItem!
    private var quitItem: NSMenuItem!
    private var statusMenu: NSMenu!
    private var launchedProcess: Process?
    private var refreshTimer: Timer?

    static func main() {
        let app = NSApplication.shared
        let delegate = ArcadiaDevelopmentLauncher()
        app.delegate = delegate
        app.setActivationPolicy(.accessory)
        app.run()
    }

    func applicationDidFinishLaunching(_ notification: Notification) {
        configureStatusItem()
        refreshMenuState()
        refreshTimer = Timer.scheduledTimer(withTimeInterval: 1.5, repeats: true) { [weak self] _ in
            self?.refreshMenuState()
        }
    }

    func applicationWillTerminate(_ notification: Notification) {
        refreshTimer?.invalidate()
        stopArcadia(wait: false)
    }

    @objc private func startArcadia() {
        guard !isArcadiaRunning() else {
            refreshMenuState()
            return
        }

        let process = makeArcadiaProcess()
        process.terminationHandler = { [weak self, weak process] _ in
            DispatchQueue.main.async {
                if self?.launchedProcess === process {
                    self?.launchedProcess = nil
                }
                self?.refreshMenuState()
            }
        }

        do {
            try process.run()
            launchedProcess = process
        } catch {
            showLaunchError(error)
        }

        refreshMenuState()
    }

    @objc private func restartArcadia() {
        guard isArcadiaRunning() else {
            refreshMenuState()
            return
        }

        stopArcadia(wait: true)
        DispatchQueue.main.asyncAfter(deadline: .now() + 0.5) { [weak self] in
            self?.startArcadia()
        }
    }

    @objc private func stopArcadiaFromMenu() {
        stopArcadia(wait: false)
        refreshMenuState()
    }

    @objc private func quitLauncher() {
        stopArcadia(wait: true)
        NSApp.terminate(nil)
    }

    func menuWillOpen(_ menu: NSMenu) {
        refreshMenuState()
    }

    private func configureStatusItem() {
        statusItem.length = NSStatusItem.squareLength
        if let button = statusItem.button {
            button.title = ""
            button.image = statusIcon()
            button.imagePosition = .imageOnly
            button.toolTip = "Arcadia Development Launcher"
        }

        let menu = NSMenu()
        menu.delegate = self
        menu.autoenablesItems = false
        startItem = NSMenuItem(title: "Start", action: #selector(startArcadia), keyEquivalent: "")
        restartItem = NSMenuItem(title: "Restart", action: #selector(restartArcadia), keyEquivalent: "")
        stopItem = NSMenuItem(title: "Stop", action: #selector(stopArcadiaFromMenu), keyEquivalent: "")
        quitItem = NSMenuItem(title: "Quit", action: #selector(quitLauncher), keyEquivalent: "q")

        for item in [startItem, restartItem, stopItem, quitItem] {
            item?.target = self
            menu.addItem(item!)
        }

        statusMenu = menu
        statusItem.menu = statusMenu
    }

    private func statusIcon() -> NSImage {
        let bundledImage = Bundle.main.url(forResource: "StatusIcon", withExtension: "png")
            .flatMap { NSImage(contentsOf: $0) }
        let image = bundledImage ?? NSApp.applicationIconImage.copy() as? NSImage ?? NSImage()
        image.size = NSSize(width: 18, height: 18)
        image.isTemplate = false
        return image
    }

    private func refreshMenuState() {
        let running = isArcadiaRunning()
        startItem?.isEnabled = !running
        restartItem?.isEnabled = running
        stopItem?.isEnabled = running
        quitItem?.isEnabled = true
    }

    private func makeArcadiaProcess() -> Process {
        let process = Process()
        let globalCommand = URL(fileURLWithPath: NSHomeDirectory())
            .appendingPathComponent(".local/bin/arcadia-gui")

        if FileManager.default.isExecutableFile(atPath: globalCommand.path) {
            process.executableURL = globalCommand
            process.arguments = []
        } else {
            process.executableURL = URL(fileURLWithPath: "/bin/bash")
            process.arguments = [
                "-lc",
                """
                export PATH="${HOME}/.cargo/bin:${PATH}"
                cd "\(repositoryRoot.path)"
                cargo build --manifest-path Desktop/Cargo.toml --target-dir target --no-default-features --features gui >/dev/null
                exec "\(repositoryRoot.path)/target/debug/arcadia"
                """
            ]
        }

        process.currentDirectoryURL = repositoryRoot
        process.environment = processEnvironment()
        attachLogFile(to: process)
        return process
    }

    private func processEnvironment() -> [String: String] {
        var environment = ProcessInfo.processInfo.environment
        let cargoBin = URL(fileURLWithPath: NSHomeDirectory()).appendingPathComponent(".cargo/bin").path
        let existingPath = environment["PATH"] ?? "/usr/bin:/bin:/usr/sbin:/sbin"
        environment["PATH"] = "\(cargoBin):\(existingPath)"
        return environment
    }

    private func attachLogFile(to process: Process) {
        let logsDirectory = URL(fileURLWithPath: NSHomeDirectory())
            .appendingPathComponent("Library/Logs/Arcadia")
        let logFile = logsDirectory.appendingPathComponent("DevelopmentLauncher.log")

        do {
            try FileManager.default.createDirectory(
                at: logsDirectory,
                withIntermediateDirectories: true
            )
            if !FileManager.default.fileExists(atPath: logFile.path) {
                FileManager.default.createFile(atPath: logFile.path, contents: nil)
            }
            let handle = try FileHandle(forWritingTo: logFile)
            try handle.seekToEnd()
            process.standardOutput = handle
            process.standardError = handle
        } catch {
            process.standardOutput = FileHandle.standardOutput
            process.standardError = FileHandle.standardError
        }
    }

    private func isArcadiaRunning() -> Bool {
        if let process = launchedProcess, process.isRunning {
            return true
        }
        return !arcadiaPIDs().isEmpty
    }

    private func stopArcadia(wait: Bool) {
        let localProcess = launchedProcess
        let pids = arcadiaPIDs()

        processQueue.async {
            if let localProcess, localProcess.isRunning {
                localProcess.terminate()
                if wait {
                    localProcess.waitUntilExit()
                }
            }

            for pid in pids {
                Darwin.kill(pid, SIGTERM)
            }

            if wait {
                self.waitForArcadiaToExit(timeout: 5.0)
            }

            DispatchQueue.main.async {
                if self.launchedProcess?.isRunning == false {
                    self.launchedProcess = nil
                }
                self.refreshMenuState()
            }
        }
    }

    private func waitForArcadiaToExit(timeout: TimeInterval) {
        let deadline = Date().addingTimeInterval(timeout)
        while Date() < deadline {
            if arcadiaPIDs().isEmpty {
                return
            }
            Thread.sleep(forTimeInterval: 0.1)
        }
    }

    private func arcadiaPIDs() -> [pid_t] {
        let process = Process()
        let pipe = Pipe()

        process.executableURL = URL(fileURLWithPath: "/bin/ps")
        process.arguments = ["-axo", "pid=,command="]
        process.standardOutput = pipe
        process.standardError = Pipe()

        do {
            try process.run()
        } catch {
            return []
        }

        process.waitUntilExit()

        let output = pipe.fileHandleForReading.readDataToEndOfFile()
        guard let text = String(data: output, encoding: .utf8) else {
            return []
        }

        let currentPID = ProcessInfo.processInfo.processIdentifier
        let binaryPath = "\(repositoryRoot.path)/target/debug/arcadia"

        return text.split(separator: "\n").compactMap { line -> pid_t? in
            let trimmed = String(line).trimmingCharacters(in: .whitespaces)
            guard let firstSpace = trimmed.firstIndex(where: { $0 == " " || $0 == "\t" }) else {
                return nil
            }

            let pidText = String(trimmed[..<firstSpace])
            let command = String(trimmed[firstSpace...]).trimmingCharacters(in: .whitespaces)

            guard let pid = pid_t(pidText), pid != currentPID else {
                return nil
            }

            if command == binaryPath || command.hasPrefix("\(binaryPath) ") {
                return pid
            }

            return nil
        }
    }

    private func showLaunchError(_ error: Error) {
        let alert = NSAlert()
        alert.messageText = "Arcadia could not be started."
        alert.informativeText = error.localizedDescription
        alert.alertStyle = .warning
        alert.runModal()
    }
}
