import SwiftUI
import UIKit

struct RemoteTarget: Codable, Identifiable {
    let ip: String
    let hostname: String
    var id: String { ip }
}

private struct SurfaceSnapshot: Codable {
    let modules: [String: Bool]
}

private struct SurfaceModulesSetPatch: Codable {
    let op: String
    let name: String
    let enabled: Bool

    init(name: String, enabled: Bool) {
        self.op = "modules_set"
        self.name = name
        self.enabled = enabled
    }
}

extension ContentView {
    func reloadModules() {
        if let route = remoteRoute {
            let json = executeCommand(
                token: "surface.snapshot",
                args: [],
                context: ExecutionContextFfi(netAs: route, netTimeoutMs: nil)
            )
            guard let data = json.data(using: .utf8),
                  let snap = try? JSONDecoder().decode(SurfaceSnapshot.self, from: data) else {
                modules = []
                return
            }
            modules = snap.modules.map { ModuleStatus(name: $0.key, enabled: $0.value) }
                .sorted { $0.name < $1.name }
        } else {
            modules = listModules().sorted { $0.name < $1.name }
        }
    }

    func refreshRemoteTargets() {
        guard isModuleEnabled(ModuleNames.lan) else {
            remoteTargets = []
            return
        }
        let json = executeCommand(
            token: "lan.session_targets",
            args: [],
            context: ExecutionContextFfi(netAs: nil, netTimeoutMs: nil)
        )
        guard let data = json.data(using: .utf8),
              let decoded = try? JSONDecoder().decode([RemoteTarget].self, from: data) else {
            remoteTargets = []
            return
        }
        remoteTargets = decoded
    }

    func updateModule(name: String, enabled: Bool) {
        moduleToggleTask?.cancel()
        moduleToggleTask = Task { @MainActor in
            do { try await Task.sleep(nanoseconds: 300_000_000) } catch { return }
            if let route = remoteRoute {
                guard let payloadData = try? JSONEncoder().encode([
                    SurfaceModulesSetPatch(name: name, enabled: enabled),
                ]),
                      let payload = String(data: payloadData, encoding: .utf8) else {
                    moduleErrorMessage = "Could not encode surface.patch payload"
                    return
                }
                let result = executeCommand(
                    token: "surface.patch",
                    args: [payload],
                    context: ExecutionContextFfi(netAs: route, netTimeoutMs: nil)
                )
                let expected = "Module \(name) \(enabled ? "enabled" : "disabled")"
                if result != expected {
                    moduleErrorMessage = result
                }
                reloadModules()
                return
            }
            if enabled {
                let probe = probeModuleToggle(name: name, enabled: true)
                if !probe.ok && !probe.missingRequirements.isEmpty {
                    pendingModuleEnable = (name: name, probe: probe)
                    showRequirementsPrompt = true
                    reloadModules()
                    return
                }
            }
            let result = setModuleEnabled(name: name, enabled: enabled)
            let expected = "Module \(name) \(enabled ? "enabled" : "disabled")"
            if result != expected {
                moduleErrorMessage = result
            }
            reloadModules()
        }
    }

    func runShellCommand() {
        let trimmed = shellCommandInput.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !trimmed.isEmpty else { return }
        let args = trimmed.split(separator: " ").map(String.init)
        shellHistory.append("$ \(trimmed)")
        let output = executeCommand(
            token: "shell.execute",
            args: args,
            context: ExecutionContextFfi(netAs: remoteRoute, netTimeoutMs: nil)
        )
        shellHistory.append(contentsOf: output.split(separator: "\n", omittingEmptySubsequences: false).map(String.init))
        shellHistory.append("")
    }

    func dismissKeyboard() {
        UIApplication.shared.sendAction(#selector(UIResponder.resignFirstResponder), to: nil, from: nil, for: nil)
    }

    /// NODE_EXEC host: transcript + reload module/nav when showing **local** host state.
    func applyRemoteMirrorSideEffects() {
        let batch = drainRemoteMirrorBatch()
        if batch.syncLocalSurface {
            refreshRemoteTargets()
            if remoteRoute == nil {
                reloadModules()
            }
        }
        if !batch.lines.isEmpty {
            shellHistory.append(contentsOf: batch.lines)
        }
    }
}
