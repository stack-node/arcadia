import SwiftUI

struct NetworkOverviewView: View {
    let theme: AppTheme
    let modules: [ModuleStatus]

    @State private var serviceInfo: LanServiceInfoFfi = LanServiceInfoFfi(
        running: false, port: 0, hostname: "", moduleEnabled: false
    )
    @State private var refreshToggle = false

    private var netEnabled: Bool {
        modules.first(where: { $0.name == ModuleNames.net })?.enabled ?? false
    }

    private var lanEnabled: Bool {
        modules.first(where: { $0.name == ModuleNames.lan })?.enabled ?? false
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 16) {
            statusCard(
                title: "Network Module",
                subtitle: "Networking foundation — required by LAN and remote session",
                enabled: netEnabled
            )

            if lanEnabled {
                lanServiceCard
            } else {
                Text("Enable the LAN module to manage the discovery service.")
                    .font(.subheadline)
                    .foregroundStyle(theme.secondaryTextColor)
            }
        }
        .onAppear { serviceInfo = lanServiceInfo() }
    }

    private var lanServiceCard: some View {
        HStack(alignment: .center, spacing: 12) {
            VStack(alignment: .leading, spacing: 4) {
                Text("LAN Discovery Service")
                    .font(.subheadline.weight(.semibold))
                    .foregroundStyle(theme.primaryTextColor)

                Text("UDP :\(serviceInfo.port) · \(serviceInfo.hostname.isEmpty ? "unknown" : serviceInfo.hostname)")
                    .font(.caption)
                    .foregroundStyle(theme.secondaryTextColor)
            }

            Spacer()

            runningBadge(serviceInfo.running)

            Button(serviceInfo.running ? "Stop" : "Start") {
                if serviceInfo.running {
                    lanStop()
                } else {
                    lanStart()
                }
                // Small delay so service thread has a chance to update the flag.
                DispatchQueue.main.asyncAfter(deadline: .now() + 0.15) {
                    serviceInfo = lanServiceInfo()
                }
            }
            .buttonStyle(.bordered)
            .font(.subheadline.weight(.semibold))
            .tint(serviceInfo.running ? .red : .green)
        }
        .padding(16)
        .background(theme.cardFillColor, in: RoundedRectangle(cornerRadius: 14, style: .continuous))
        .overlay {
            RoundedRectangle(cornerRadius: 14, style: .continuous)
                .stroke(theme.cardStrokeColor, lineWidth: 1)
        }
    }

    private func statusCard(title: String, subtitle: String, enabled: Bool) -> some View {
        HStack(alignment: .center, spacing: 12) {
            VStack(alignment: .leading, spacing: 4) {
                Text(title)
                    .font(.subheadline.weight(.semibold))
                    .foregroundStyle(theme.primaryTextColor)
                Text(subtitle)
                    .font(.caption)
                    .foregroundStyle(theme.secondaryTextColor)
                    .fixedSize(horizontal: false, vertical: true)
            }
            Spacer()
            enabledBadge(enabled)
        }
        .padding(16)
        .background(theme.cardFillColor, in: RoundedRectangle(cornerRadius: 14, style: .continuous))
        .overlay {
            RoundedRectangle(cornerRadius: 14, style: .continuous)
                .stroke(theme.cardStrokeColor, lineWidth: 1)
        }
    }

    private func enabledBadge(_ enabled: Bool) -> some View {
        Text(enabled ? "enabled" : "disabled")
            .font(.caption.weight(.semibold))
            .padding(.horizontal, 8)
            .padding(.vertical, 3)
            .background(enabled ? Color.green.opacity(0.2) : Color.gray.opacity(0.2))
            .foregroundStyle(enabled ? Color.green : Color.gray)
            .clipShape(Capsule())
    }

    private func runningBadge(_ running: Bool) -> some View {
        Text(running ? "running" : "stopped")
            .font(.caption.weight(.semibold))
            .padding(.horizontal, 8)
            .padding(.vertical, 3)
            .background(running ? Color.green.opacity(0.2) : Color.red.opacity(0.2))
            .foregroundStyle(running ? Color.green : Color.red)
            .clipShape(Capsule())
    }
}
