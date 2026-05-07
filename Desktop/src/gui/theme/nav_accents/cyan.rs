use super::palette::{rgb8, NavAccentPalette};

pub(super) fn palette(is_dark: bool) -> NavAccentPalette {
    if is_dark {
        NavAccentPalette {
            icon_idle: rgb8(14, 116, 144),
            icon_active: rgb8(34, 211, 238),
            row_selected: rgb8(21, 42, 48),
            row_hover: rgb8(26, 53, 64),
        }
    } else {
        NavAccentPalette {
            icon_idle: rgb8(8, 145, 178),
            icon_active: rgb8(8, 145, 178),
            row_selected: rgb8(236, 254, 255),
            row_hover: rgb8(207, 250, 254),
        }
    }
}
