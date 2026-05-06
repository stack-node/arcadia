use super::palette::{rgb8, NavAccentPalette};

pub(super) fn palette(is_dark: bool) -> NavAccentPalette {
    if is_dark {
        NavAccentPalette {
            icon_idle: rgb8(4, 120, 87),
            icon_active: rgb8(52, 211, 153),
            row_selected: rgb8(20, 41, 34),
            row_hover: rgb8(26, 51, 40),
        }
    } else {
        NavAccentPalette {
            icon_idle: rgb8(4, 120, 87),
            icon_active: rgb8(5, 150, 105),
            row_selected: rgb8(236, 253, 245),
            row_hover: rgb8(209, 250, 229),
        }
    }
}
