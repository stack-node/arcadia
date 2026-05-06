import SwiftUI
import UIKit

struct PageDefinition: Identifiable, Decodable {
    let id: String
    let title: String
    let description: String
    let glyph: String
    let systemImage: String

    enum CodingKeys: String, CodingKey {
        case id
        case title
        case description
        case glyph
        case systemImage = "system_image"
    }
}

struct GroupDefinition: Identifiable, Decodable {
    let id: String
    let label: String
    let glyph: String
    let systemImage: String
    let pageIDs: [String]

    enum CodingKeys: String, CodingKey {
        case id
        case label
        case glyph
        case systemImage = "system_image"
        case pageIDs = "pages"
    }
}

struct NavigationRegistry: Decodable {
    let pages: [PageDefinition]
    let groups: [GroupDefinition]
    let globalPages: [String]
    let defaultGroup: String
    let defaultPage: String

    enum CodingKeys: String, CodingKey {
        case pages
        case groups
        case globalPages = "global_pages"
        case defaultGroup = "default_group"
        case defaultPage = "default_page"
    }
}

struct ContentView: View {
    @Environment(\.colorScheme) private var colorScheme

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
    @State private var shellCommandInput = "echo hello"
    @State private var shellHistory: [String] = ["Arcadia Terminal ready."]

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

            sidebar
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
            if open {
                dismissKeyboard()
            }
        }
        .alert("Enable with requirements?", isPresented: $showRequirementsPrompt, presenting: pendingModuleEnable) { pending in
            Button("Cancel", role: .cancel) {
                pendingModuleEnable = nil
            }
            Button("Enable") {
                _ = setModuleEnabledWithRequirements(name: pending.name, enabled: true)
                pendingModuleEnable = nil
                reloadModules()
            }
        } message: { pending in
            Text("To enable \(pending.name), Arcadia needs to enable: \(pending.probe.missingRequirements.joined(separator: ", ")). Continue with --requirements?")
        }
    }

    private var sidebarOffset: CGFloat {
        if isSidebarOpen {
            return min(0, sidebarDragOffset)
        }
        return max(-sidebarWidth, -sidebarWidth + max(0, sidebarDragOffset))
    }

    private var closeSidebarGesture: some Gesture {
        DragGesture(minimumDistance: 12)
            .onChanged { value in
                guard isSidebarOpen else { return }
                sidebarDragOffset = min(0, value.translation.width)
            }
            .onEnded { value in
                guard isSidebarOpen else { return }
                let closingSwipe = value.translation.width < -sidebarSwipeThreshold
                    || value.predictedEndTranslation.width < -sidebarSwipeThreshold
                withAnimation(.spring(response: 0.42, dampingFraction: 0.88)) {
                    isSidebarOpen = !closingSwipe
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
                let openingSwipe = value.translation.width > sidebarSwipeThreshold
                    || value.predictedEndTranslation.width > sidebarSwipeThreshold
                withAnimation(.spring(response: 0.42, dampingFraction: 0.88)) {
                    isSidebarOpen = openingSwipe
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

    private var activePage: PageDefinition {
        if isPageVisible(activePageID), let page = registry.pages.first(where: { $0.id == activePageID }) {
            return page
        }
        if let firstVisible = registry.pages.first(where: { isPageVisible($0.id) }) {
            return firstVisible
        }
        return registry.pages[0]
    }

    private var activeGroup: GroupDefinition {
        if let group = visibleGroups.first(where: { $0.id == activeGroupID }) {
            return group
        }
        return visibleGroups.first ?? registry.groups[0]
    }

    private var activeGroupPages: [PageDefinition] {
        activeGroup.pageIDs.filter { isPageVisible($0) }.compactMap(pageDefinition)
    }

    private var shellEnabled: Bool {
        modules.first(where: { $0.name == "shell" })?.enabled ?? false
    }

    private var visibleGroups: [GroupDefinition] {
        registry.groups.filter { group in
            group.pageIDs.contains { isPageVisible($0) }
        }
    }

    private func pageDefinition(_ pageID: String) -> PageDefinition? {
        registry.pages.first(where: { $0.id == pageID })
    }

    private func isPageVisible(_ pageID: String) -> Bool {
        if pageID == "utility.shell" {
            return shellEnabled
        }
        return true
    }

    private func selectGroup(_ groupID: String) {
        activeGroupID = groupID
        if let group = visibleGroups.first(where: { $0.id == groupID }),
           let firstPageID = group.pageIDs.first(where: { isPageVisible($0) }) {
            activePageID = firstPageID
        }
    }

    private func selectPage(_ pageID: String) {
        activePageID = pageID
        if pageID == "global.modules" {
            reloadModules()
        }
    }

    private static func loadNavigationRegistry() -> NavigationRegistry {
        let fallback = NavigationRegistry(
            pages: [
                PageDefinition(id: "utility.shell", title: "Shell", description: "Run and manage shell utility actions.", glyph: "SH", systemImage: "terminal"),
                PageDefinition(id: "global.dashboard", title: "Dashboard", description: "Overview of the Arcadia application surface.", glyph: "DH", systemImage: "house"),
                PageDefinition(id: "global.logs", title: "Logs", description: "Recent logs and activity stream appear here.", glyph: "LG", systemImage: "doc.text.magnifyingglass"),
                PageDefinition(id: "global.settings", title: "Settings", description: "App preferences and configuration controls appear here.", glyph: "ST", systemImage: "gearshape")
                ,
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

    private var isDarkMode: Bool {
        colorScheme == .dark
    }

    private var primaryTextColor: Color {
        isDarkMode ? .white : Color.black.opacity(0.85)
    }

    private var secondaryTextColor: Color {
        isDarkMode ? .white.opacity(0.72) : Color.black.opacity(0.62)
    }

    private var tertiaryTextColor: Color {
        isDarkMode ? .white.opacity(0.54) : Color.black.opacity(0.5)
    }

    private var accentTextColor: Color {
        isDarkMode ? .white.opacity(0.92) : Color.black.opacity(0.82)
    }

    private var selectedTextColor: Color {
        isDarkMode ? .white : Color.black.opacity(0.88)
    }

    private var cardFillColor: Color {
        isDarkMode ? .white.opacity(0.08) : .white.opacity(0.72)
    }

    private var cardStrokeColor: Color {
        isDarkMode ? .white.opacity(0.14) : Color.black.opacity(0.1)
    }

    private var selectedFillColor: Color {
        isDarkMode ? .white.opacity(0.12) : Color.black.opacity(0.08)
    }

    private var selectedStrokeColor: Color {
        isDarkMode ? .white.opacity(0.18) : Color.black.opacity(0.16)
    }

    private var sidebarShadowColor: Color {
        isDarkMode ? .black.opacity(0.28) : .black.opacity(0.12)
    }

    private var contentShadowColor: Color {
        isDarkMode ? .black.opacity(0.22) : .black.opacity(0.08)
    }

    private var glassBackground: some View {
        ZStack {
            if isDarkMode {
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
                .fill(isDarkMode ? Color.white.opacity(0.18) : Color.white.opacity(0.6))
                .frame(width: 320)
                .blur(radius: 70)
                .offset(x: -120, y: -250)

            Circle()
                .fill(isDarkMode ? Color.cyan.opacity(0.16) : Color.cyan.opacity(0.2))
                .frame(width: 360)
                .blur(radius: 90)
                .offset(x: 150, y: -160)

            Circle()
                .fill(isDarkMode ? Color.blue.opacity(0.22) : Color.blue.opacity(0.16))
                .frame(width: 380)
                .blur(radius: 110)
                .offset(x: 130, y: 260)
        }
        .ignoresSafeArea()
    }

    private var mainContent: some View {
        NavigationStack {
            VStack(alignment: .leading, spacing: 24) {
                if activePage.id != "utility.shell" {
                    HStack(alignment: .top) {
                        VStack(alignment: .leading, spacing: 10) {
                            Text(activePage.title)
                                .font(.system(size: 40, weight: .semibold, design: .rounded))
                                .foregroundStyle(primaryTextColor)

                            Text(activePage.description)
                                .font(.body)
                                .foregroundStyle(secondaryTextColor)
                                .fixedSize(horizontal: false, vertical: true)
                        }

                        Spacer()

                        Image(systemName: sidebarSymbol)
                            .font(.system(size: 28, weight: .medium))
                            .foregroundStyle(accentTextColor)
                            .frame(width: 58, height: 58)
                            .background(cardFillColor, in: RoundedRectangle(cornerRadius: 18, style: .continuous))
                            .overlay {
                                RoundedRectangle(cornerRadius: 18, style: .continuous)
                                    .stroke(cardStrokeColor, lineWidth: 1)
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
                    .fill(cardFillColor)
                    .background(
                        RoundedRectangle(cornerRadius: 30, style: .continuous)
                            .fill(.ultraThinMaterial)
                    )
                    .overlay {
                        RoundedRectangle(cornerRadius: 30, style: .continuous)
                            .stroke(cardStrokeColor, lineWidth: 1)
                    }
                    .shadow(color: contentShadowColor, radius: 40, x: 0, y: 18)
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
                            .foregroundStyle(accentTextColor)
                            .frame(width: 52, height: 52)
                            .background(.ultraThinMaterial, in: Circle())
                            .overlay {
                                Circle()
                                    .fill(isDarkMode ? .white.opacity(0.04) : .white.opacity(0.35))
                            }
                            .overlay {
                                Circle()
                                    .stroke(cardStrokeColor, lineWidth: 1)
                            }
                            .shadow(color: contentShadowColor, radius: 14, x: 0, y: 8)
                    }
                    .buttonStyle(.plain)
                    .accessibilityLabel(isSidebarOpen ? "Close sidebar" : "Open sidebar")
                }
            }
            .toolbarBackground(.hidden, for: .navigationBar)
        }
    }

    private var sidebar: some View {
        VStack(alignment: .leading, spacing: 14) {
            VStack(alignment: .leading, spacing: 6) {
                Text("Arcadia")
                    .font(.system(size: 28, weight: .semibold, design: .rounded))
                    .foregroundStyle(primaryTextColor)

                Text("Liquid glass")
                    .font(.subheadline)
                    .foregroundStyle(secondaryTextColor)
            }
            .padding(.horizontal, 22)
            .padding(.top, 28)
            .padding(.bottom, 10)

            Text("Groups")
                .font(.caption.weight(.semibold))
                .foregroundStyle(tertiaryTextColor)
                .padding(.horizontal, 16)

            ScrollView(.horizontal, showsIndicators: false) {
                HStack(spacing: 8) {
                    ForEach(visibleGroups) { group in
                        Button {
                            selectGroup(group.id)
                        } label: {
                            VStack(spacing: 6) {
                                Image(systemName: group.systemImage)
                                    .font(.system(size: 14, weight: .semibold))
                                Text(group.label)
                            }
                                .font(.caption.weight(activeGroupID == group.id ? .semibold : .medium))
                                .foregroundStyle(activeGroupID == group.id ? selectedTextColor : secondaryTextColor)
                                .frame(width: 64, height: 64)
                                .background(
                                    RoundedRectangle(cornerRadius: 12, style: .continuous)
                                        .fill(activeGroupID == group.id ? selectedFillColor : .clear)
                                )
                                .overlay {
                                    RoundedRectangle(cornerRadius: 12, style: .continuous)
                                        .stroke(activeGroupID == group.id ? selectedStrokeColor : .clear, lineWidth: 1)
                                }
                        }
                        .buttonStyle(.plain)
                    }
                }
                .padding(.horizontal, 14)
            }

            ScrollView(.vertical, showsIndicators: false) {
                VStack(alignment: .leading, spacing: 8) {
                    Text(activeGroup.label)
                        .font(.caption.weight(.semibold))
                        .foregroundStyle(tertiaryTextColor)
                        .padding(.top, 2)
                        .padding(.horizontal, 16)

                    ForEach(activeGroupPages) { page in
                        pageButton(page: page)
                    }
                }
                .padding(.bottom, 8)
            }
            .frame(maxHeight: .infinity)

            Text("Global")
                .font(.caption.weight(.semibold))
                .foregroundStyle(tertiaryTextColor)
                .padding(.horizontal, 16)

            ForEach(registry.globalPages, id: \.self) { pageID in
                if let page = pageDefinition(pageID) {
                    pageButton(page: page)
                }
            }
            .padding(.bottom, 14)
        }
        .frame(width: sidebarWidth)
        .frame(maxHeight: .infinity, alignment: .topLeading)
        .background(.ultraThinMaterial)
        .background(isDarkMode ? .white.opacity(0.05) : .white.opacity(0.3))
        .overlay {
            RoundedRectangle(cornerRadius: 0)
                .stroke(cardStrokeColor, lineWidth: 1)
        }
        .shadow(color: sidebarShadowColor, radius: 28, x: 8, y: 0)
        .ignoresSafeArea()
    }

    private var sidebarSymbol: String {
        activePage.systemImage
    }

    private var contentBody: some View {
        VStack(spacing: 16) {
            if activePage.id == "global.modules" {
                modulesCard
            } else if activePage.id == "utility.shell" {
                shellCard
            } else {
                glassCard(
                    title: "Primary Surface",
                    subtitle: "This page is rendered from shared page definitions."
                )

                HStack(spacing: 16) {
                    glassMetric(title: "Sidebar", value: isSidebarOpen ? "Open" : "Closed")
                    glassMetric(title: "Selection", value: activePage.title)
                }
            }
        }
    }

    private var shellCard: some View {
        VStack(spacing: 0) {
            HStack {
                Text("Terminal")
                    .font(.headline)
                    .foregroundStyle(primaryTextColor)
                Spacer()
                Button("Clear") {
                    shellHistory.removeAll()
                }
                .buttonStyle(.bordered)
            }
            .padding(12)
            .background(cardFillColor)

            ScrollView {
                VStack(alignment: .leading, spacing: 6) {
                    ForEach(Array(shellHistory.enumerated()), id: \.offset) { _, line in
                        Text(line)
                            .font(.system(.caption, design: .monospaced))
                            .foregroundStyle(accentTextColor)
                            .frame(maxWidth: .infinity, alignment: .leading)
                            .textSelection(.enabled)
                    }
                }
                .padding(12)
            }
            .frame(maxWidth: .infinity, maxHeight: .infinity)

            HStack(spacing: 8) {
                Text("$")
                    .font(.system(.body, design: .monospaced))
                    .foregroundStyle(secondaryTextColor)
                TextField("Type a command", text: $shellCommandInput)
                    .textFieldStyle(.plain)
                    .font(.system(.body, design: .monospaced))
                    .foregroundStyle(primaryTextColor)
                    .onSubmit { runShellCommand() }
                Button("Run") { runShellCommand() }
                    .buttonStyle(.borderedProminent)
            }
            .padding(12)
            .background(cardFillColor)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
        .background(cardFillColor, in: RoundedRectangle(cornerRadius: 24, style: .continuous))
        .overlay {
            RoundedRectangle(cornerRadius: 24, style: .continuous)
                .stroke(cardStrokeColor, lineWidth: 1)
        }
    }

    private var modulesCard: some View {
        glassCard(title: "Global Modules", subtitle: "Enable or disable modules for all surfaces.") {
            VStack(alignment: .leading, spacing: 10) {
                ForEach(modules, id: \.name) { module in
                    HStack {
                        VStack(alignment: .leading, spacing: 4) {
                            Text(module.name)
                                .font(.body.weight(.semibold))
                                .foregroundStyle(primaryTextColor)
                            Text(module.enabled ? "Enabled" : "Disabled")
                                .font(.caption)
                                .foregroundStyle(secondaryTextColor)
                        }
                        Spacer()
                        Toggle("", isOn: Binding(
                            get: { module.enabled },
                            set: { newValue in
                                updateModule(name: module.name, enabled: newValue)
                            }
                        ))
                        .labelsHidden()
                    }
                    .padding(.vertical, 6)
                }
            }
        }
    }

    private func reloadModules() {
        modules = listModules().sorted { $0.name < $1.name }
    }

    private func updateModule(name: String, enabled: Bool) {
        if enabled {
            let probe = probeModuleToggle(name: name, enabled: true)
            if !probe.ok && !probe.missingRequirements.isEmpty {
                pendingModuleEnable = (name: name, probe: probe)
                showRequirementsPrompt = true
                reloadModules()
                return
            }
        }
        _ = setModuleEnabled(name: name, enabled: enabled)
        reloadModules()
    }

    private func runShellCommand() {
        let trimmed = shellCommandInput.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !trimmed.isEmpty else {
            return
        }
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

    private func pageButton(page: PageDefinition) -> some View {
        Button {
            selectPage(page.id)
        } label: {
            HStack(spacing: 12) {
                Image(systemName: page.systemImage)
                    .font(.system(size: 16, weight: .semibold))
                    .frame(width: 20)
                Text(page.title)
                    .frame(maxWidth: .infinity, alignment: .leading)
            }
            .font(.body.weight(activePageID == page.id ? .semibold : .medium))
            .foregroundStyle(activePageID == page.id ? selectedTextColor : secondaryTextColor)
            .padding(.horizontal, 16)
            .frame(height: 50)
            .background(
                RoundedRectangle(cornerRadius: 16, style: .continuous)
                    .fill(activePageID == page.id ? selectedFillColor : .clear)
            )
            .overlay {
                RoundedRectangle(cornerRadius: 16, style: .continuous)
                    .stroke(activePageID == page.id ? selectedStrokeColor : .clear, lineWidth: 1)
            }
        }
        .buttonStyle(.plain)
        .padding(.horizontal, 14)
    }

    private func glassCard(title: String, subtitle: String) -> some View {
        glassCard(title: title, subtitle: subtitle) {
            EmptyView()
        }
    }

    private func glassCard<Content: View>(title: String, subtitle: String, @ViewBuilder content: () -> Content) -> some View {
        VStack(alignment: .leading, spacing: 8) {
            Text(title)
                .font(.headline)
                .foregroundStyle(primaryTextColor)

            Text(subtitle)
                .font(.subheadline)
                .foregroundStyle(secondaryTextColor)

            content()
        }
        .frame(maxWidth: .infinity, alignment: .leading)
        .padding(20)
        .background(cardFillColor, in: RoundedRectangle(cornerRadius: 24, style: .continuous))
        .overlay {
            RoundedRectangle(cornerRadius: 24, style: .continuous)
                .stroke(cardStrokeColor, lineWidth: 1)
        }
    }

    private func glassMetric(title: String, value: String) -> some View {
        VStack(alignment: .leading, spacing: 8) {
            Text(title.uppercased())
                .font(.caption.weight(.semibold))
                .foregroundStyle(tertiaryTextColor)

            Text(value)
                .font(.title3.weight(.semibold))
                .foregroundStyle(primaryTextColor)
                .lineLimit(1)
                .minimumScaleFactor(0.8)
        }
        .frame(maxWidth: .infinity, minHeight: 108, alignment: .topLeading)
        .padding(18)
        .background(cardFillColor, in: RoundedRectangle(cornerRadius: 22, style: .continuous))
        .overlay {
            RoundedRectangle(cornerRadius: 22, style: .continuous)
                .stroke(cardStrokeColor, lineWidth: 1)
        }
    }
}
