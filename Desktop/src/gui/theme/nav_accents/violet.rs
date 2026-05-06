use super::palette::{rgb8, NavAccentPalette};

pub(super) fn palette(is_dark: bool) -> NavAccentPalette {
    if is_dark {
        NavAccentPalette {
            icon_idle: rgb8(109, 40, 217),
            icon_active: rgb8(167, 139, 250),
            row_selected: rgb8(37, 26, 51),
            row_hover: rgb8(46, 33, 64),
        }
    } else {
        NavAccentPalette {
            icon_idle: rgb8(109, 40, 217),
            icon_active: rgb8(124, 58, 237),
            row_selected: rgb8(245, 243, 255),
            row_hover: rgb8(237, 233, 254),
        }
    }
}
