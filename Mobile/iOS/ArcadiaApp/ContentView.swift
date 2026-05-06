import SwiftUI

struct SidebarItem: Identifiable, Hashable {
    let id = UUID()
    let title: String
    let systemImage: String
}

struct ContentView: View {
    @Environment(\.horizontalSizeClass) private var horizontalSizeClass

    private let sidebarItems = [
        SidebarItem(title: "Dashboard", systemImage: "square.grid.2x2"),
        SidebarItem(title: "Shell", systemImage: "terminal"),
        SidebarItem(title: "Modules", systemImage: "switch.2"),
        SidebarItem(title: "Settings", systemImage: "gearshape")
    ]

    @State private var isSidebarOpen = true
    @State private var selectedItemTitle = "Dashboard"
    @State private var coreRuntimeEnabled = true
    @State private var commandRouterEnabled = true
    @State private var remoteBridgeEnabled = false
    @State private var telemetryEnabled = false
    @State private var safeguardsEnabled = true
    @State private var autoUpdatesEnabled = true
    @State private var selectedEnvironment = "Production"

    var body: some View {
        ZStack(alignment: .leading) {
            glassBackground

            mainContent
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
                .offset(x: isSidebarOpen ? 0 : -300)
        }
        .preferredColorScheme(.dark)
        .animation(.spring(response: 0.42, dampingFraction: 0.88), value: isSidebarOpen)
    }

    private var glassBackground: some View {
        ZStack {
            LinearGradient(
                colors: [
                    Color(red: 0.05, green: 0.08, blue: 0.16),
                    Color(red: 0.07, green: 0.14, blue: 0.25),
                    Color(red: 0.03, green: 0.06, blue: 0.12)
                ],
                startPoint: .topLeading,
                endPoint: .bottomTrailing
            )

            Circle()
                .fill(Color.white.opacity(0.18))
                .frame(width: 320)
                .blur(radius: 70)
                .offset(x: -120, y: -250)

            Circle()
                .fill(Color.cyan.opacity(0.16))
                .frame(width: 360)
                .blur(radius: 90)
                .offset(x: 150, y: -160)

            Circle()
                .fill(Color.blue.opacity(0.22))
                .frame(width: 380)
                .blur(radius: 110)
                .offset(x: 130, y: 260)
        }
        .ignoresSafeArea()
    }

    private var mainContent: some View {
        NavigationStack {
            VStack(alignment: .leading, spacing: 24) {
                HStack(alignment: .top) {
                    VStack(alignment: .leading, spacing: 10) {
                        Text(selectedItemTitle)
                            .font(.system(size: 40, weight: .semibold, design: .rounded))
                            .foregroundStyle(.white)

                        Text("A liquid glass shell with translucent navigation and polished placeholder surfaces.")
                            .font(.body)
                            .foregroundStyle(.white.opacity(0.72))
                            .fixedSize(horizontal: false, vertical: true)
                    }

                    Spacer()

                    Image(systemName: sidebarSymbol)
                        .font(.system(size: 28, weight: .medium))
                        .foregroundStyle(.white.opacity(0.86))
                        .frame(width: 58, height: 58)
                        .background(.white.opacity(0.08), in: RoundedRectangle(cornerRadius: 18, style: .continuous))
                        .overlay {
                            RoundedRectangle(cornerRadius: 18, style: .continuous)
                                .stroke(.white.opacity(0.14), lineWidth: 1)
                        }
                }

                contentBody

                Spacer(minLength: 0)
            }
            .padding(24)
            .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
            .background {
                RoundedRectangle(cornerRadius: 30, style: .continuous)
                    .fill(.white.opacity(0.08))
                    .background(
                        RoundedRectangle(cornerRadius: 30, style: .continuous)
                            .fill(.ultraThinMaterial)
                    )
                    .overlay {
                        RoundedRectangle(cornerRadius: 30, style: .continuous)
                            .stroke(.white.opacity(0.16), lineWidth: 1)
                    }
                    .shadow(color: .black.opacity(0.22), radius: 40, x: 0, y: 18)
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
                            .foregroundStyle(.white.opacity(0.92))
                            .frame(width: 52, height: 52)
                            .background(.ultraThinMaterial, in: Circle())
                            .overlay {
                                Circle()
                                    .fill(.white.opacity(0.04))
                            }
                            .overlay {
                                Circle()
                                    .stroke(.white.opacity(0.08), lineWidth: 1)
                            }
                            .shadow(color: .black.opacity(0.18), radius: 14, x: 0, y: 8)
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
                    .foregroundStyle(.white)

                Text("Liquid glass")
                    .font(.subheadline)
                    .foregroundStyle(.white.opacity(0.64))
            }
            .padding(.horizontal, 22)
            .padding(.top, 28)
            .padding(.bottom, 10)

            ForEach(sidebarItems) { item in
                Button {
                    selectedItemTitle = item.title
                } label: {
                    HStack(spacing: 12) {
                        Image(systemName: item.systemImage)
                            .font(.system(size: 16, weight: .semibold))
                            .frame(width: 20)
                        Text(item.title)
                            .frame(maxWidth: .infinity, alignment: .leading)
                    }
                    .font(.body.weight(selectedItemTitle == item.title ? .semibold : .medium))
                    .foregroundStyle(selectedItemTitle == item.title ? .white : .white.opacity(0.78))
                    .padding(.horizontal, 16)
                    .frame(height: 50)
                    .background(
                        RoundedRectangle(cornerRadius: 16, style: .continuous)
                            .fill(selectedItemTitle == item.title ? .white.opacity(0.12) : .clear)
                    )
                    .overlay {
                        RoundedRectangle(cornerRadius: 16, style: .continuous)
                            .stroke(selectedItemTitle == item.title ? .white.opacity(0.18) : .clear, lineWidth: 1)
                    }
                }
                .buttonStyle(.plain)
                .padding(.horizontal, 14)
            }

            Spacer()

            VStack(alignment: .leading, spacing: 8) {
                Text("Ambient")
                    .font(.caption.weight(.semibold))
                    .foregroundStyle(.white.opacity(0.54))

                Text("A minimal app shell ready for real navigation.")
                    .font(.footnote)
                    .foregroundStyle(.white.opacity(0.72))
            }
            .padding(18)
            .background(.white.opacity(0.07), in: RoundedRectangle(cornerRadius: 18, style: .continuous))
            .overlay {
                RoundedRectangle(cornerRadius: 18, style: .continuous)
                    .stroke(.white.opacity(0.12), lineWidth: 1)
            }
            .padding(.horizontal, 16)
            .padding(.bottom, 26)
        }
        .frame(width: 292)
        .frame(maxHeight: .infinity, alignment: .topLeading)
        .background(.ultraThinMaterial)
        .background(.white.opacity(0.05))
        .overlay {
            RoundedRectangle(cornerRadius: 0)
                .stroke(.white.opacity(0.1), lineWidth: 1)
        }
        .shadow(color: .black.opacity(0.28), radius: 28, x: 8, y: 0)
        .ignoresSafeArea()
    }

    private var sidebarSymbol: String {
        sidebarItems.first(where: { $0.title == selectedItemTitle })?.systemImage ?? "square.grid.2x2"
    }

    @ViewBuilder
    private var contentBody: some View {
        if selectedItemTitle == "Modules" {
            modulesPage
        } else {
            VStack(spacing: 16) {
                glassCard(
                    title: "Primary Surface",
                    subtitle: "Use this area for the first real destination you add."
                )

                HStack(spacing: 16) {
                    glassMetric(title: "Sidebar", value: isSidebarOpen ? "Open" : "Closed")
                    glassMetric(title: "Selection", value: selectedItemTitle)
                }
            }
        }
    }

    private var modulesPage: some View {
        ScrollView(showsIndicators: false) {
            VStack(spacing: 16) {
                if isCompactLayout {
                    VStack(spacing: 16) {
                        glassMetric(title: "Active", value: "\(activeModuleCount)")
                        glassMetric(title: "Environment", value: selectedEnvironment)
                    }
                } else {
                    HStack(spacing: 16) {
                        glassMetric(title: "Active", value: "\(activeModuleCount)")
                        glassMetric(title: "Environment", value: selectedEnvironment)
                    }
                }

                glassCard(title: "Runtime Modules", subtitle: "Separate core services from optional integrations and make state obvious.") {
                    VStack(spacing: 12) {
                        moduleRow(
                            title: "Core Runtime",
                            subtitle: "Required for local orchestration and state management.",
                            isOn: $coreRuntimeEnabled,
                            accent: .cyan
                        )
                        moduleRow(
                            title: "Command Router",
                            subtitle: "Dispatches actions between shell, modules, and system tools.",
                            isOn: $commandRouterEnabled,
                            accent: .blue
                        )
                        moduleRow(
                            title: "Remote Bridge",
                            subtitle: "Allows outbound connections to paired services and agents.",
                            isOn: $remoteBridgeEnabled,
                            accent: .mint
                        )
                    }
                }

                if isCompactLayout {
                    VStack(spacing: 16) {
                        safetyCard
                        releaseChannelCard
                    }
                } else {
                    HStack(alignment: .top, spacing: 16) {
                        safetyCard
                        releaseChannelCard
                    }
                }
            }
            .frame(maxWidth: .infinity, alignment: .leading)
            .padding(.bottom, 8)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .top)
    }

    private var activeModuleCount: Int {
        [
            coreRuntimeEnabled,
            commandRouterEnabled,
            remoteBridgeEnabled,
            telemetryEnabled,
            safeguardsEnabled,
            autoUpdatesEnabled
        ].filter { $0 }.count
    }

    private var isCompactLayout: Bool {
        horizontalSizeClass != .regular
    }

    private var safetyCard: some View {
        glassCard(title: "Safety", subtitle: "High-risk capabilities should have explicit switches.") {
            VStack(spacing: 12) {
                moduleRow(
                    title: "Safeguards",
                    subtitle: "Blocks destructive operations until they are explicitly allowed.",
                    isOn: $safeguardsEnabled,
                    accent: .green
                )
                moduleRow(
                    title: "Telemetry",
                    subtitle: "Collects session diagnostics and performance traces.",
                    isOn: $telemetryEnabled,
                    accent: .orange
                )
            }
        }
    }

    private var releaseChannelCard: some View {
        glassCard(title: "Release Channel", subtitle: "Pick where modules resolve from and how they update.") {
            VStack(alignment: .leading, spacing: 14) {
                Picker("Environment", selection: $selectedEnvironment) {
                    Text("Prod").tag("Production")
                    Text("Stage").tag("Staging")
                    Text("Local").tag("Local")
                }
                .pickerStyle(.segmented)

                Toggle(isOn: $autoUpdatesEnabled) {
                    VStack(alignment: .leading, spacing: 4) {
                        Text("Automatic updates")
                            .foregroundStyle(.white)
                        Text("Refresh module manifests and compatibility rules.")
                            .font(.footnote)
                            .foregroundStyle(.white.opacity(0.68))
                            .fixedSize(horizontal: false, vertical: true)
                    }
                }
                .tint(.white.opacity(0.9))
            }
        }
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
                .foregroundStyle(.white)

            Text(subtitle)
                .font(.subheadline)
                .foregroundStyle(.white.opacity(0.72))

            content()
        }
        .frame(maxWidth: .infinity, alignment: .leading)
        .padding(20)
        .background(.white.opacity(0.08), in: RoundedRectangle(cornerRadius: 24, style: .continuous))
        .overlay {
            RoundedRectangle(cornerRadius: 24, style: .continuous)
                .stroke(.white.opacity(0.14), lineWidth: 1)
        }
    }

    private func moduleRow(title: String, subtitle: String, isOn: Binding<Bool>, accent: Color) -> some View {
        HStack(alignment: .center, spacing: 14) {
            Circle()
                .fill(accent.opacity(isOn.wrappedValue ? 0.95 : 0.35))
                .frame(width: 10, height: 10)
                .shadow(color: accent.opacity(isOn.wrappedValue ? 0.6 : 0), radius: 8)

            VStack(alignment: .leading, spacing: 4) {
                Text(title)
                    .foregroundStyle(.white)
                    .font(.body.weight(.semibold))

                Text(subtitle)
                    .foregroundStyle(.white.opacity(0.68))
                    .font(.footnote)
                    .fixedSize(horizontal: false, vertical: true)
            }

            Spacer(minLength: 12)

            Toggle("", isOn: isOn)
                .labelsHidden()
                .tint(.white.opacity(0.92))
        }
        .padding(14)
        .background(.white.opacity(0.06), in: RoundedRectangle(cornerRadius: 18, style: .continuous))
        .overlay {
            RoundedRectangle(cornerRadius: 18, style: .continuous)
                .stroke(.white.opacity(0.08), lineWidth: 1)
        }
    }

    private func glassMetric(title: String, value: String) -> some View {
        VStack(alignment: .leading, spacing: 8) {
            Text(title.uppercased())
                .font(.caption.weight(.semibold))
                .foregroundStyle(.white.opacity(0.52))

            Text(value)
                .font(.title3.weight(.semibold))
                .foregroundStyle(.white)
                .lineLimit(1)
                .minimumScaleFactor(0.8)
        }
        .frame(maxWidth: .infinity, minHeight: 108, alignment: .topLeading)
        .padding(18)
        .background(.white.opacity(0.08), in: RoundedRectangle(cornerRadius: 22, style: .continuous))
        .overlay {
            RoundedRectangle(cornerRadius: 22, style: .continuous)
                .stroke(.white.opacity(0.14), lineWidth: 1)
        }
    }
}
