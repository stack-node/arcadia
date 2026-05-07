import SwiftUI

struct GlassCard<Content: View>: View {
    @Environment(\.colorScheme) private var colorScheme
    private var theme: AppTheme { AppTheme(isDark: colorScheme == .dark) }

    let title: String
    let subtitle: String
    let content: Content

    init(title: String, subtitle: String, @ViewBuilder content: () -> Content) {
        self.title = title
        self.subtitle = subtitle
        self.content = content()
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            Text(title)
                .font(.headline)
                .foregroundStyle(theme.primaryTextColor)

            Text(subtitle)
                .font(.subheadline)
                .foregroundStyle(theme.secondaryTextColor)

            content
        }
        .frame(maxWidth: .infinity, alignment: .leading)
        .padding(20)
        .background(theme.cardFillColor, in: RoundedRectangle(cornerRadius: 24, style: .continuous))
        .overlay {
            RoundedRectangle(cornerRadius: 24, style: .continuous)
                .stroke(theme.cardStrokeColor, lineWidth: 1)
        }
    }
}

extension GlassCard where Content == EmptyView {
    init(title: String, subtitle: String) {
        self.init(title: title, subtitle: subtitle) { EmptyView() }
    }
}

struct GlassMetric: View {
    @Environment(\.colorScheme) private var colorScheme
    private var theme: AppTheme { AppTheme(isDark: colorScheme == .dark) }

    let title: String
    let value: String

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            Text(title.uppercased())
                .font(.caption.weight(.semibold))
                .foregroundStyle(theme.tertiaryTextColor)

            Text(value)
                .font(.title3.weight(.semibold))
                .foregroundStyle(theme.primaryTextColor)
                .lineLimit(1)
                .minimumScaleFactor(0.8)
        }
        .frame(maxWidth: .infinity, minHeight: 108, alignment: .topLeading)
        .padding(18)
        .background(theme.cardFillColor, in: RoundedRectangle(cornerRadius: 22, style: .continuous))
        .overlay {
            RoundedRectangle(cornerRadius: 22, style: .continuous)
                .stroke(theme.cardStrokeColor, lineWidth: 1)
        }
    }
}
