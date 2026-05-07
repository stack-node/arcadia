import Foundation
import SwiftUI
import UIKit

struct RemoteTarget: Codable, Identifiable {
    let ip: String
    let hostname: String
    var id: String { ip }
}

private struct SurfacePayload: Codable {
    let modules: [String: Bool]
    let revision: UInt64?
    let extra: SurfaceExtra?
}

private struct SurfaceExtra: Codable {
    let navigationRegistry: NavigationRegistry?

    enum CodingKeys: String, CodingKey {
        case navigationRegistry = "navigation_registry"
    }
}

private struct SurfaceModulesSetPatch: Codable {
    let op: String
    let name: String
    let enabled: Bool
    let clientId: String?

    enum CodingKeys: String, CodingKey {
        case op, name, enabled
        case clientId = "client_id"
    }

    init(name: String, enabled: Bool, clientId: String?) {
        self.op = "modules_set"
        self.name = name
        self.enabled = enabled
        self.clientId = clientId
    }
}

extension ContentView {
    /// ARCADIA_NET_AS env beats thin-client.toml; shape matches ExecutionContext.net_as (`lan:host`).
    func applyThinClientBootstrapRoute() {
        let env = ProcessInfo.processInfo.environment["ARCADIA_NET_AS"]?
            .trimmingCharacters(in: .whitespacesAndNewlines)
        let persisted = thinClientPreferredRouteGet()?
            .trimmingCharacters(in: .whitespacesAndNewlines)
        let raw: String?
        if let e = env, !e.isEmpty { raw = e }
        else if let p = persisted, !p.isEmpty { raw = p }
        else { raw = nil }
        guard let r = raw,
              isModuleEnabled(ModuleNames.lan),
              isModuleEnabled(ModuleNames.remoteSession) else { return }
        let route = r.hasPrefix("lan:") ? r : "lan:\(r)"
        remoteRoute = route
    }

    func ensureActiveNavigationSelection() {
        let pageIds = Set(navigationRegistry.pages.map(\.id))
        let groupIds = Set(navigationRegistry.groups.map(\.id))
        if !groupIds.contains(activeGroupID) {
            activeGroupID = navigationRegistry.defaultGroup
        }
        if !pageIds.contains(activePageID) {
            activePageID = navigationRegistry.defaultPage
        }
    }

    func reloadModules() {
        if let route = remoteRoute {
            let json = executeCommand(
                token: "surface.snapshot",
                args: [],
                context: ExecutionContextFfi(netAs: route, netTimeoutMs: nil)
            )
            guard let data = json.data(using: .utf8),
                  let payload = try? JSONDecoder().decode(SurfacePayload.self, from: data) else {
                modules = []
                return
            }
            modules = payload.modules.map { ModuleStatus(name: $0.key, enabled: $0.value) }
                .sorted { $0.name < $1.name }
            if let nav = payload.extra?.navigationRegistry, !nav.pages.isEmpty, !nav.groups.isEmpty {
                navigationRegistry = nav
            }
            ensureActiveNavigationSelection()
        } else {
            modules = listModules().sorted { $0.name < $1.name }
            navigationRegistry = Self.loadNavigationRegistry()
            ensureActiveNavigationSelection()
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
                    SurfaceModulesSetPatch(
                        name: name,
                        enabled: enabled,
                        clientId: thinClientSurfaceClientId()
                    ),
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
