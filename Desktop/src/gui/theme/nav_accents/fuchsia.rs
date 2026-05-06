use super::palette::{rgb8, NavAccentPalette};

pub(super) fn palette(is_dark: bool) -> NavAccentPalette {
    if is_dark {
        NavAccentPalette {
            icon_idle: rgb8(162, 28, 175),
            icon_active: rgb8(232, 121, 249),
            row_selected: rgb8(45, 21, 51),
            row_hover: rgb8(56, 26, 64),
        }
    } else {
        NavAccentPalette {
            icon_idle: rgb8(162, 28, 175),
            icon_active: rgb8(192, 38, 211),
            row_selected: rgb8(253, 244, 255),
            row_hover: rgb8(250, 232, 255),
        }
    }
}
