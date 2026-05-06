import SwiftUI

struct ModulesView: View {
    @Environment(\.colorScheme) private var colorScheme
    private var theme: AppTheme { AppTheme(isDark: colorScheme == .dark) }

    let modules: [ModuleStatus]
    let onToggle: (String, Bool) -> Void
    let onAppear: () -> Void

    var body: some View {
        GlassCard(title: "Global Modules", subtitle: "Enable or disable modules for all surfaces.") {
            VStack(alignment: .leading, spacing: 10) {
                ForEach(modules, id: \.name) { module in
                    HStack {
                        VStack(alignment: .leading, spacing: 4) {
                            Text(module.name)
                                .font(.body.weight(.semibold))
                                .foregroundStyle(theme.primaryTextColor)
                            Text(module.enabled ? "Enabled" : "Disabled")
                                .font(.caption)
                                .foregroundStyle(theme.secondaryTextColor)
                        }
                        Spacer()
                        Toggle("", isOn: Binding(
                            get: { module.enabled },
                            set: { newValue in onToggle(module.name, newValue) }
                        ))
                        .labelsHidden()
                    }
                    .padding(.vertical, 6)
                }
            }
        }
        .onAppear(perform: onAppear)
    }
}
