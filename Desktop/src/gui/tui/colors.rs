use gpui::Rgba;

pub fn default_fg(is_dark: bool) -> Rgba {
    if is_dark {
        Rgba {
            r: 0.87,
            g: 0.87,
            b: 0.87,
            a: 1.0,
        }
    } else {
        Rgba {
            r: 0.07,
            g: 0.07,
            b: 0.07,
            a: 1.0,
        }
    }
}

pub fn default_bg(is_dark: bool) -> Rgba {
    if is_dark {
        Rgba {
            r: 0.047,
            g: 0.047,
            b: 0.047,
            a: 1.0,
        }
    } else {
        Rgba {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        }
    }
}

pub fn rgba_eq(a: Rgba, b: Rgba) -> bool {
    a.r == b.r && a.g == b.g && a.b == b.b && a.a == b.a
}

/// Convert a vt100 Color to an Rgba value.
pub fn vt_color(color: vt100::Color, is_fg: bool, is_dark: bool) -> Rgba {
    match color {
        vt100::Color::Default => {
            if is_fg {
                default_fg(is_dark)
            } else {
                default_bg(is_dark)
            }
        }
        vt100::Color::Idx(idx) => ansi_indexed(idx),
        vt100::Color::Rgb(r, g, b) => Rgba {
            r: r as f32 / 255.0,
            g: g as f32 / 255.0,
            b: b as f32 / 255.0,
            a: 1.0,
        },
    }
}

fn ansi_indexed(idx: u8) -> Rgba {
    let (r, g, b): (u8, u8, u8) = match idx {
        0 => (0x1c, 0x1c, 0x1c),
        1 => (0xcc, 0x00, 0x00),
        2 => (0x4e, 0x9a, 0x06),
        3 => (0xc4, 0xa0, 0x00),
        4 => (0x34, 0x65, 0xa4),
        5 => (0x75, 0x50, 0x7b),
        6 => (0x06, 0x98, 0x9a),
        7 => (0xd3, 0xd7, 0xcf),
        8 => (0x55, 0x57, 0x53),
        9 => (0xef, 0x29, 0x29),
        10 => (0x8a, 0xe2, 0x34),
        11 => (0xfc, 0xe9, 0x4f),
        12 => (0x72, 0x9f, 0xcf),
        13 => (0xad, 0x7f, 0xa8),
        14 => (0x34, 0xe2, 0xe2),
        15 => (0xee, 0xee, 0xec),
        16..=231 => {
            let n = idx - 16;
            let bi = n % 6;
            let gi = (n / 6) % 6;
            let ri = n / 36;
            let c = |v: u8| if v == 0 { 0u8 } else { 55 + v * 40 };
            return Rgba {
                r: c(ri) as f32 / 255.0,
                g: c(gi) as f32 / 255.0,
                b: c(bi) as f32 / 255.0,
                a: 1.0,
            };
        }
        _ => {
            let v = 8u8.saturating_add((idx - 232).saturating_mul(10));
            return Rgba {
                r: v as f32 / 255.0,
                g: v as f32 / 255.0,
                b: v as f32 / 255.0,
                a: 1.0,
            };
        }
    };
    Rgba {
        r: r as f32 / 255.0,
        g: g as f32 / 255.0,
        b: b as f32 / 255.0,
        a: 1.0,
    }
}
