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

    // MARK: - Navigation accents (mirrors `Desktop/src/gui/theme.rs` `nav_accent_palette`)

    struct NavAccentPalette {
        let iconIdle: Color
        let iconActive: Color
        let selectedFill: Color
        let hoverFill: Color
    }

    func navAccentPalette(_ key: String) -> NavAccentPalette {
        switch key {
        case "amber":
            return isDark
                ? navRgb(idle: (180, 83, 9), active: (251, 191, 36), selected: (53, 42, 28), hover: (63, 52, 40))
                : navRgb(idle: (180, 83, 9), active: (217, 119, 6), selected: (255, 251, 235), hover: (254, 243, 199))
        case "cyan":
            return isDark
                ? navRgb(idle: (14, 116, 144), active: (34, 211, 238), selected: (21, 42, 48), hover: (26, 53, 64))
                : navRgb(idle: (8, 145, 178), active: (8, 145, 178), selected: (236, 254, 255), hover: (207, 250, 254))
        case "emerald":
            return isDark
                ? navRgb(idle: (4, 120, 87), active: (52, 211, 153), selected: (20, 41, 34), hover: (26, 51, 40))
                : navRgb(idle: (4, 120, 87), active: (5, 150, 105), selected: (236, 253, 245), hover: (209, 250, 229))
        case "violet":
            return isDark
                ? navRgb(idle: (109, 40, 217), active: (167, 139, 250), selected: (37, 26, 51), hover: (46, 33, 64))
                : navRgb(idle: (109, 40, 217), active: (124, 58, 237), selected: (245, 243, 255), hover: (237, 233, 254))
        case "orange":
            return isDark
                ? navRgb(idle: (194, 65, 12), active: (251, 146, 60), selected: (51, 24, 16), hover: (64, 34, 24))
                : navRgb(idle: (194, 65, 12), active: (234, 88, 12), selected: (255, 247, 237), hover: (255, 237, 213))
        case "indigo":
            return isDark
                ? navRgb(idle: (67, 56, 202), active: (129, 140, 248), selected: (30, 27, 51), hover: (37, 33, 64))
                : navRgb(idle: (67, 56, 202), active: (79, 70, 229), selected: (238, 242, 255), hover: (224, 231, 255))
        case "fuchsia":
            return isDark
                ? navRgb(idle: (162, 28, 175), active: (232, 121, 249), selected: (45, 21, 51), hover: (56, 26, 64))
                : navRgb(idle: (162, 28, 175), active: (192, 38, 211), selected: (253, 244, 255), hover: (250, 232, 255))
        case "teal":
            return isDark
                ? navRgb(idle: (15, 118, 110), active: (45, 212, 191), selected: (20, 40, 36), hover: (26, 51, 46))
                : navRgb(idle: (15, 118, 110), active: (13, 148, 136), selected: (240, 253, 250), hover: (204, 251, 241))
        case "sky":
            return isDark
                ? navRgb(idle: (148, 163, 184), active: (147, 197, 253), selected: (31, 42, 62), hover: (36, 50, 70))
                : navRgb(idle: (107, 114, 128), active: (29, 78, 216), selected: (225, 231, 255), hover: (238, 242, 255))
        default:
            return navAccentPalette("sky")
        }
    }

    private func navRgb(
        idle: (Int, Int, Int),
        active: (Int, Int, Int),
        selected: (Int, Int, Int),
        hover: (Int, Int, Int)
    ) -> NavAccentPalette {
        func c(_ t: (Int, Int, Int)) -> Color {
            Color(red: Double(t.0) / 255, green: Double(t.1) / 255, blue: Double(t.2) / 255)
        }
        return NavAccentPalette(iconIdle: c(idle), iconActive: c(active), selectedFill: c(selected), hoverFill: c(hover))
    }
}
