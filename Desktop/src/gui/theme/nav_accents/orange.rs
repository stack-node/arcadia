use super::palette::{rgb8, NavAccentPalette};

pub(super) fn palette(is_dark: bool) -> NavAccentPalette {
    if is_dark {
        NavAccentPalette {
            icon_idle: rgb8(194, 65, 12),
            icon_active: rgb8(251, 146, 60),
            row_selected: rgb8(51, 24, 16),
            row_hover: rgb8(64, 34, 24),
        }
    } else {
        NavAccentPalette {
            icon_idle: rgb8(194, 65, 12),
            icon_active: rgb8(234, 88, 12),
            row_selected: rgb8(255, 247, 237),
            row_hover: rgb8(255, 237, 213),
        }
    }
}
