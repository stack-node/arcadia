use super::palette::{rgb8, NavAccentPalette};

pub(super) fn palette(is_dark: bool) -> NavAccentPalette {
    if is_dark {
        NavAccentPalette {
            icon_idle: rgb8(15, 118, 110),
            icon_active: rgb8(45, 212, 191),
            row_selected: rgb8(20, 40, 36),
            row_hover: rgb8(26, 51, 46),
        }
    } else {
        NavAccentPalette {
            icon_idle: rgb8(15, 118, 110),
            icon_active: rgb8(13, 148, 136),
            row_selected: rgb8(240, 253, 250),
            row_hover: rgb8(204, 251, 241),
        }
    }
}
