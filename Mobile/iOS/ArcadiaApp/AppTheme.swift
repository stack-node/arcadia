import SwiftUI

struct AppTheme {
    let isDark: Bool

    var primaryTextColor: Color   { isDark ? .white : Color.black.opacity(0.85) }
    var secondaryTextColor: Color { isDark ? .white.opacity(0.72) : Color.black.opacity(0.62) }
    var tertiaryTextColor: Color  { isDark ? .white.opacity(0.54) : Color.black.opacity(0.5) }
    var accentTextColor: Color    { isDark ? .white.opacity(0.92) : Color.black.opacity(0.82) }
    var selectedTextColor: Color  { isDark ? .white : Color.black.opacity(0.88) }

    var cardFillColor: Color     { isDark ? .white.opacity(0.08) : .white.opacity(0.72) }
    var cardStrokeColor: Color   { isDark ? .white.opacity(0.14) : Color.black.opacity(0.1) }
    var selectedFillColor: Color { isDark ? .white.opacity(0.12) : Color.black.opacity(0.08) }
    var selectedStrokeColor: Color { isDark ? .white.opacity(0.18) : Color.black.opacity(0.16) }

    var sidebarShadowColor: Color  { isDark ? .black.opacity(0.28) : .black.opacity(0.12) }
    var contentShadowColor: Color  { isDark ? .black.opacity(0.22) : .black.opacity(0.08) }
}
