import Combine
import SwiftUI
import UIKit

struct ContentView: View {
    @Environment(\.colorScheme) var colorScheme
    var theme: AppTheme { AppTheme(isDark: colorScheme == .dark) }

    let sidebarWidth: CGFloat = 292
    let sidebarSwipeThreshold: CGFloat = 80

    let registry: NavigationRegistry

    @State var showSplash = true
    @State var isSidebarOpen = true
    @State var activeGroupID: String
    @State var activePageID: String
    @State var sidebarDragOffset: CGFloat = 0
    @State var modules: [ModuleStatus] = []
    @State var pendingModuleEnable: (name: String, probe: ModuleToggleResult)?
    @State var showRequirementsPrompt = false
    @State var shellCommandInput = ""
    @State var shellHistory: [String] = ["Arcadia Terminal ready."]
    @State var moduleToggleTask: Task<Void, Never>?
    @State var moduleErrorMessage: String?
    @State var remoteRoute: String?
    @State var remoteTargets: [RemoteTarget] = []

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
                isPageVisible: { pageID in self.isPageVisible(pageID) },
                remoteSessionEnabled: isModuleEnabled(ModuleNames.remoteSession),
                remoteRoute: $remoteRoute,
                remoteTargets: remoteTargets,
                refreshRemoteTargets: refreshRemoteTargets,
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
        .overlay {
            if showSplash {
                SplashView {
                    withAnimation(.easeOut(duration: 0.2)) {
                        showSplash = false
                    }
                }
                .ignoresSafeArea()
                .transition(.opacity)
            }
        }
        .onAppear {
            refreshRemoteTargets()
            reloadModules()
        }
        .onReceive(Timer.publish(every: 0.25, on: .main, in: .common).autoconnect()) { _ in
            applyRemoteMirrorSideEffects()
        }
        .onChange(of: remoteRoute) { _, _ in
            reloadModules()
        }
        .onChange(of: isSidebarOpen) { open in
            if open { dismissKeyboard() }
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
}
