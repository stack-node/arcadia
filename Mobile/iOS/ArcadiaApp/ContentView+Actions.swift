import SwiftUI
import UIKit

extension ContentView {
    func reloadModules() {
        modules = listModules().sorted { $0.name < $1.name }
    }

    func updateModule(name: String, enabled: Bool) {
        moduleToggleTask?.cancel()
        moduleToggleTask = Task { @MainActor in
            do { try await Task.sleep(nanoseconds: 300_000_000) } catch { return }
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
            context: ExecutionContextFfi(netAs: nil, netTimeoutMs: nil)
        )
        shellHistory.append(contentsOf: output.split(separator: "\n", omittingEmptySubsequences: false).map(String.init))
        shellHistory.append("")
    }

    func dismissKeyboard() {
        UIApplication.shared.sendAction(#selector(UIResponder.resignFirstResponder), to: nil, from: nil, for: nil)
    }
}
