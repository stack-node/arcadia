import SwiftUI
import UIKit

struct ContentView: View {
    @Environment(\.colorScheme) private var colorScheme
    private var theme: AppTheme { AppTheme(isDark: colorScheme == .dark) }

    private let sidebarWidth: CGFloat = 292
    private let sidebarSwipeThreshold: CGFloat = 80

    private let registry: NavigationRegistry

    @State private var isSidebarOpen = true
    @State private var activeGroupID: String
    @State private var activePageID: String
    @State private var sidebarDragOffset: CGFloat = 0
    @State private var modules: [ModuleStatus] = []
    @State private var pendingModuleEnable: (name: String, probe: ModuleToggleResult)?
    @State private var showRequirementsPrompt = false
    @State private var shellCommandInput = ""
    @State private var shellHistory: [String] = ["Arcadia Terminal ready."]
    @State private var moduleToggleTask: Task<Void, Never>?
    @State private var moduleErrorMessage: String?

    init() {
        let loadedRegistry = Self.loadNavigationRegistry()
        self.registry = loadedRegistry
        _activeGroupID = State(initialValue: loadedRegistry.defaultGroup)
        _activePageID = State(initialValue: loadedRegistry.defaultPage)
    }

    var body: some View {
        ZStack(alignment: .leading) {
            glassBackground

            mainContent
                .simultaneousGesture(closeSidebarGesture)
                .overlay {
                    if isSidebarOpen {
                        Rectangle()
                            .fill(.black.opacity(0.12))
                            .background(.ultraThinMaterial.opacity(0.25))
                            .ignoresSafeArea()
                            .transition(.opacity)
                            .onTapGesture {
                                withAnimation(.easeInOut(duration: 0.26)) {
                                    isSidebarOpen = false
                                }
                            }
                    }
                }

            SidebarView(
                registry: registry,
                sidebarWidth: sidebarWidth,
                sidebarSwipeThreshold: sidebarSwipeThreshold,
                shellEnabled: shellEnabled,
                activeGroupID: $activeGroupID,
                activePageID: $activePageID
            )
            .offset(x: sidebarOffset)
            .gesture(closeSidebarGesture)

            if !isSidebarOpen {
                edgeSwipeHandle
            }
        }
        .animation(.spring(response: 0.42, dampingFraction: 0.88), value: isSidebarOpen)
        .animation(.interactiveSpring(response: 0.3, dampingFraction: 0.9), value: sidebarDragOffset)
        .onAppear {
            reloadModules()
        }
        .onChange(of: isSidebarOpen) { open in
            if open { dismissKeyboard() }
        }
        .onChange(of: activePageID) { pageID in
            if pageID == "global.modules" { reloadModules() }
        }
        .alert("Enable with requirements?", isPresented: $showRequirementsPrompt, presenting: pendingModuleEnable) { pending in
            Button("Cancel", role: .cancel) { pendingModuleEnable = nil }
            Button("Enable") {
                let result = setModuleEnabledWithRequirements(name: pending.name, enabled: true)
                if result != "Module \(pending.name) enabled" {
                    moduleErrorMessage = result
                }
                pendingModuleEnable = nil
                reloadModules()
            }
        } message: { pending in
            Text("To enable \(pending.name), Arcadia needs to enable: \(pending.probe.missingRequirements.joined(separator: ", ")). Continue with --requirements?")
        }
        .alert("Module Error", isPresented: Binding(
            get: { moduleErrorMessage != nil },
            set: { if !$0 { moduleErrorMessage = nil } }
        )) {
            Button("OK", role: .cancel) { moduleErrorMessage = nil }
        } message: {
            Text(moduleErrorMessage ?? "")
        }
    }

    // MARK: - Sidebar geometry

    private var sidebarOffset: CGFloat {
        isSidebarOpen
            ? min(0, sidebarDragOffset)
            : max(-sidebarWidth, -sidebarWidth + max(0, sidebarDragOffset))
    }

    private var closeSidebarGesture: some Gesture {
        DragGesture(minimumDistance: 12)
            .onChanged { value in
                guard isSidebarOpen else { return }
                sidebarDragOffset = min(0, value.translation.width)
            }
            .onEnded { value in
                guard isSidebarOpen else { return }
                let closing = value.translation.width < -sidebarSwipeThreshold
                    || value.predictedEndTranslation.width < -sidebarSwipeThreshold
                withAnimation(.spring(response: 0.42, dampingFraction: 0.88)) {
                    isSidebarOpen = !closing
                    sidebarDragOffset = 0
                }
            }
    }

    private var openSidebarGesture: some Gesture {
        DragGesture(minimumDistance: 12)
            .onChanged { value in
                guard !isSidebarOpen else { return }
                sidebarDragOffset = max(0, value.translation.width)
            }
            .onEnded { value in
                guard !isSidebarOpen else { return }
                let opening = value.translation.width > sidebarSwipeThreshold
                    || value.predictedEndTranslation.width > sidebarSwipeThreshold
                withAnimation(.spring(response: 0.42, dampingFraction: 0.88)) {
                    isSidebarOpen = opening
                    sidebarDragOffset = 0
                }
            }
    }

    private var edgeSwipeHandle: some View {
        Rectangle()
            .fill(.clear)
            .frame(width: 24)
            .contentShape(Rectangle())
            .ignoresSafeArea()
            .gesture(openSidebarGesture)
    }

    // MARK: - Navigation state

    private var shellEnabled: Bool {
        modules.first(where: { $0.name == "shell" })?.enabled ?? false
    }

    private func isPageVisible(_ pageID: String) -> Bool {
        pageID == "utility.shell" ? shellEnabled : true
    }

    private var activePage: PageDefinition {
        if isPageVisible(activePageID), let page = registry.pages.first(where: { $0.id == activePageID }) {
            return page
        }
        if let firstVisible = registry.pages.first(where: { isPageVisible($0.id) }) {
            return firstVisible
        }
        return registry.pages[0]
    }

    // MARK: - Content

    private var mainContent: some View {
        NavigationStack {
            VStack(alignment: .leading, spacing: 24) {
                if activePage.id != "utility.shell" {
                    HStack(alignment: .top) {
                        VStack(alignment: .leading, spacing: 10) {
                            Text(activePage.title)
                                .font(.system(size: 40, weight: .semibold, design: .rounded))
                                .foregroundStyle(theme.primaryTextColor)

                            Text(activePage.description)
                                .font(.body)
                                .foregroundStyle(theme.secondaryTextColor)
                                .fixedSize(horizontal: false, vertical: true)
                        }

                        Spacer()

                        Image(systemName: activePage.systemImage)
                            .font(.system(size: 28, weight: .medium))
                            .foregroundStyle(theme.accentTextColor)
                            .frame(width: 58, height: 58)
                            .background(theme.cardFillColor, in: RoundedRectangle(cornerRadius: 18, style: .continuous))
                            .overlay {
                                RoundedRectangle(cornerRadius: 18, style: .continuous)
                                    .stroke(theme.cardStrokeColor, lineWidth: 1)
                            }
                    }
                }

                contentBody

                Spacer(minLength: 0)
            }
            .padding(24)
            .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
            .background {
                RoundedRectangle(cornerRadius: 30, style: .continuous)
                    .fill(theme.cardFillColor)
                    .background(
                        RoundedRectangle(cornerRadius: 30, style: .continuous)
                            .fill(.ultraThinMaterial)
                    )
                    .overlay {
                        RoundedRectangle(cornerRadius: 30, style: .continuous)
                            .stroke(theme.cardStrokeColor, lineWidth: 1)
                    }
                    .shadow(color: theme.contentShadowColor, radius: 40, x: 0, y: 18)
            }
            .padding(.horizontal, 18)
            .padding(.vertical, 10)
            .toolbar {
                ToolbarItem(placement: .topBarLeading) {
                    Button {
                        withAnimation(.spring(response: 0.42, dampingFraction: 0.88)) {
                            isSidebarOpen.toggle()
                        }
                    } label: {
                        Image(systemName: "sidebar.leading")
                            .font(.system(size: 18, weight: .semibold))
                            .foregroundStyle(theme.accentTextColor)
                            .frame(width: 52, height: 52)
                            .background(.ultraThinMaterial, in: Circle())
                            .overlay {
                                Circle()
                                    .fill(colorScheme == .dark ? .white.opacity(0.04) : .white.opacity(0.35))
                            }
                            .overlay {
                                Circle()
                                    .stroke(theme.cardStrokeColor, lineWidth: 1)
                            }
                            .shadow(color: theme.contentShadowColor, radius: 14, x: 0, y: 8)
                    }
                    .buttonStyle(.plain)
                    .accessibilityLabel(isSidebarOpen ? "Close sidebar" : "Open sidebar")
                }
            }
            .toolbarBackground(.hidden, for: .navigationBar)
        }
    }

    @ViewBuilder
    private var contentBody: some View {
        if activePage.id == "global.modules" {
            ModulesView(modules: modules, onToggle: updateModule)
        } else if activePage.id == "utility.shell" {
            ShellView(shellHistory: $shellHistory, shellCommandInput: $shellCommandInput, onRun: runShellCommand)
        } else {
            VStack(spacing: 16) {
                GlassCard(title: "Primary Surface", subtitle: "This page is rendered from shared page definitions.")
                HStack(spacing: 16) {
                    GlassMetric(title: "Sidebar", value: isSidebarOpen ? "Open" : "Closed")
                    GlassMetric(title: "Selection", value: activePage.title)
                }
            }
        }
    }

    private var glassBackground: some View {
        ZStack {
            if colorScheme == .dark {
                LinearGradient(
                    colors: [
                        Color(red: 0.05, green: 0.08, blue: 0.16),
                        Color(red: 0.07, green: 0.14, blue: 0.25),
                        Color(red: 0.03, green: 0.06, blue: 0.12)
                    ],
                    startPoint: .topLeading,
                    endPoint: .bottomTrailing
                )
            } else {
                LinearGradient(
                    colors: [
                        Color(red: 0.95, green: 0.97, blue: 1.0),
                        Color(red: 0.92, green: 0.96, blue: 0.99),
                        Color(red: 0.97, green: 0.98, blue: 1.0)
                    ],
                    startPoint: .topLeading,
                    endPoint: .bottomTrailing
                )
            }

            Circle()
                .fill(colorScheme == .dark ? Color.white.opacity(0.18) : Color.white.opacity(0.6))
                .frame(width: 320)
                .blur(radius: 70)
                .offset(x: -120, y: -250)

            Circle()
                .fill(colorScheme == .dark ? Color.cyan.opacity(0.16) : Color.cyan.opacity(0.2))
                .frame(width: 360)
                .blur(radius: 90)
                .offset(x: 150, y: -160)

            Circle()
                .fill(colorScheme == .dark ? Color.blue.opacity(0.22) : Color.blue.opacity(0.16))
                .frame(width: 380)
                .blur(radius: 110)
                .offset(x: 130, y: 260)
        }
        .ignoresSafeArea()
    }

    // MARK: - Actions

    private func reloadModules() {
        modules = listModules().sorted { $0.name < $1.name }
    }

    private func updateModule(name: String, enabled: Bool) {
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

    private func runShellCommand() {
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

    private func dismissKeyboard() {
        UIApplication.shared.sendAction(#selector(UIResponder.resignFirstResponder), to: nil, from: nil, for: nil)
    }

    // MARK: - Navigation registry

    private static func loadNavigationRegistry() -> NavigationRegistry {
        let fallback = NavigationRegistry(
            pages: [
                PageDefinition(id: "utility.shell", title: "Shell", description: "Run and manage shell utility actions.", glyph: "SH", systemImage: "terminal"),
                PageDefinition(id: "global.dashboard", title: "Dashboard", description: "Overview of the Arcadia application surface.", glyph: "DH", systemImage: "house"),
                PageDefinition(id: "global.logs", title: "Logs", description: "Recent logs and activity stream appear here.", glyph: "LG", systemImage: "doc.text.magnifyingglass"),
                PageDefinition(id: "global.settings", title: "Settings", description: "App preferences and configuration controls appear here.", glyph: "ST", systemImage: "gearshape"),
                PageDefinition(id: "global.modules", title: "Modules", description: "Manage global module availability and dependency requirements.", glyph: "MD", systemImage: "switch.2")
            ],
            groups: [
                GroupDefinition(id: "utilities", label: "Utilities", glyph: "UT", systemImage: "wrench.and.screwdriver", pageIDs: ["utility.shell"])
            ],
            globalPages: ["global.dashboard", "global.logs", "global.settings", "global.modules"],
            defaultGroup: "utilities",
            defaultPage: "global.dashboard"
        )

        let payload = navigationRegistryJson()
        guard let data = payload.data(using: .utf8),
              let decoded = try? JSONDecoder().decode(NavigationRegistry.self, from: data),
              !decoded.pages.isEmpty,
              !decoded.groups.isEmpty else {
            return fallback
        }
        return decoded
    }
}
