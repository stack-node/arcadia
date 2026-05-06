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

    static let splashBackgroundTop = Color(red: 0.060, green: 0.055, blue: 0.580)
    static let splashBackgroundMid = Color(red: 0.205, green: 0.105, blue: 0.760)
    static let splashBackgroundHorizon = Color(red: 0.790, green: 0.240, blue: 0.760)
    static let splashBackgroundBottom = Color(red: 1.000, green: 0.480, blue: 0.560)

    static let splashHorizonPink = Color(red: 1.000, green: 0.250, blue: 0.670)
    static let splashHorizonGold = Color(red: 1.000, green: 0.690, blue: 0.250)

    static let splashHillBack = Color(red: 0.430, green: 0.150, blue: 0.900)
    static let splashHillLeft = Color(red: 0.500, green: 0.180, blue: 0.920)
    static let splashHillRight = Color(red: 0.420, green: 0.135, blue: 0.835)
    static let splashHillFront = Color(red: 0.060, green: 0.050, blue: 0.520)

    static let splashArchCore = Color(red: 0.930, green: 0.860, blue: 1.000)
    static let splashArchGlow = Color(red: 0.765, green: 0.610, blue: 1.000)
    static let splashStar = Color.white

    static let splashSunLayers: [(radius: Double, alpha: Double, color: Color)] = [
        (4.2, 0.055, Color(red: 1.000, green: 0.300, blue: 0.620)),
        (3.2, 0.115, Color(red: 1.000, green: 0.500, blue: 0.280)),
        (2.2, 0.210, Color(red: 1.000, green: 0.690, blue: 0.300)),
        (1.45, 0.440, Color(red: 1.000, green: 0.830, blue: 0.520)),
        (1.0, 1.000, Color(red: 1.000, green: 0.950, blue: 0.770))
    ]
}
