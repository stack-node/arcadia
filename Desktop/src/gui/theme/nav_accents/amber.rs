use super::palette::{rgb8, NavAccentPalette};

pub(super) fn palette(is_dark: bool) -> NavAccentPalette {
    if is_dark {
        NavAccentPalette {
            icon_idle: rgb8(180, 83, 9),
            icon_active: rgb8(251, 191, 36),
            row_selected: rgb8(53, 42, 28),
            row_hover: rgb8(63, 52, 40),
        }
    } else {
        NavAccentPalette {
            icon_idle: rgb8(180, 83, 9),
            icon_active: rgb8(217, 119, 6),
            row_selected: rgb8(255, 251, 235),
            row_hover: rgb8(254, 243, 199),
        }
    }
}
