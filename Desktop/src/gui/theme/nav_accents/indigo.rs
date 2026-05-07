use super::palette::{rgb8, NavAccentPalette};

pub(super) fn palette(is_dark: bool) -> NavAccentPalette {
    if is_dark {
        NavAccentPalette {
            icon_idle: rgb8(67, 56, 202),
            icon_active: rgb8(129, 140, 248),
            row_selected: rgb8(30, 27, 51),
            row_hover: rgb8(37, 33, 64),
        }
    } else {
        NavAccentPalette {
            icon_idle: rgb8(67, 56, 202),
            icon_active: rgb8(79, 70, 229),
            row_selected: rgb8(238, 242, 255),
            row_hover: rgb8(224, 231, 255),
        }
    }
}
