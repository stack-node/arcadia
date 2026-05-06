use super::palette::{rgb8, NavAccentPalette};

pub(super) fn palette(is_dark: bool) -> NavAccentPalette {
    if is_dark {
        NavAccentPalette {
            icon_idle: rgb8(148, 163, 184),
            icon_active: rgb8(147, 197, 253),
            row_selected: rgb8(31, 42, 62),
            row_hover: rgb8(36, 50, 70),
        }
    } else {
        NavAccentPalette {
            icon_idle: rgb8(107, 114, 128),
            icon_active: rgb8(29, 78, 216),
            row_selected: rgb8(225, 231, 255),
            row_hover: rgb8(238, 242, 255),
        }
    }
}
