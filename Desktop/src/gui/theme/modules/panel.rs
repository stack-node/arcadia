use gpui::Rgba;

pub fn module_panel_bg(is_dark: bool) -> Rgba {
    if is_dark {
        Rgba {
            r: 0.095,
            g: 0.115,
            b: 0.145,
            a: 1.0,
        }
    } else {
        Rgba {
            r: 0.965,
            g: 0.978,
            b: 0.995,
            a: 1.0,
        }
    }
}

pub fn module_panel_stroke(is_dark: bool) -> Rgba {
    if is_dark {
        Rgba {
            r: 0.190,
            g: 0.230,
            b: 0.290,
            a: 1.0,
        }
    } else {
        Rgba {
            r: 0.840,
            g: 0.885,
            b: 0.945,
            a: 1.0,
        }
    }
}
