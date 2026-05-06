use gpui::{Svg, svg};

pub fn icon_path(glyph_key: &'static str) -> &'static str {
    match glyph_key {
        "terminal" => "icons/terminal.svg",
        "home" => "icons/home.svg",
        "logs" => "icons/logs.svg",
        "settings" => "icons/settings.svg",
        "modules" => "icons/modules.svg",
        "tools" => "icons/tools.svg",
        _ => "icons/terminal.svg",
    }
}

pub fn render_icon(glyph_key: &'static str) -> Svg {
    svg().path(icon_path(glyph_key))
}
