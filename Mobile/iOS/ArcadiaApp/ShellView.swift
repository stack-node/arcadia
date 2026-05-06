import SwiftUI

struct ShellView: View {
    @Environment(\.colorScheme) private var colorScheme
    private var theme: AppTheme { AppTheme(isDark: colorScheme == .dark) }

    @Binding var shellHistory: [String]
    @Binding var shellCommandInput: String
    let onRun: () -> Void

    var body: some View {
        VStack(spacing: 0) {
            HStack {
                Text("Terminal")
                    .font(.headline)
                    .foregroundStyle(theme.primaryTextColor)
                Spacer()
                Button("Clear") {
                    shellHistory.removeAll()
                }
                .buttonStyle(.bordered)
            }
            .padding(12)
            .background(theme.cardFillColor)

            ScrollView {
                VStack(alignment: .leading, spacing: 6) {
                    ForEach(Array(shellHistory.enumerated()), id: \.offset) { _, line in
                        Text(line)
                            .font(.system(.caption, design: .monospaced))
                            .foregroundStyle(theme.accentTextColor)
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
                    .foregroundStyle(theme.secondaryTextColor)
                TextField("Type a command", text: $shellCommandInput)
                    .textFieldStyle(.plain)
                    .font(.system(.body, design: .monospaced))
                    .foregroundStyle(theme.primaryTextColor)
                    .onSubmit { onRun() }
                Button("Run") { onRun() }
                    .buttonStyle(.borderedProminent)
            }
            .padding(12)
            .background(theme.cardFillColor)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
        .background(theme.cardFillColor, in: RoundedRectangle(cornerRadius: 24, style: .continuous))
        .overlay {
            RoundedRectangle(cornerRadius: 24, style: .continuous)
                .stroke(theme.cardStrokeColor, lineWidth: 1)
        }
    }
}
