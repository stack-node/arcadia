pub fn render_glyph(glyph_key: &'static str) -> &'static str {
    match glyph_key {
        "group_one" => "◼",
        "group_two" => "◻",
        "grid" => "⊞",
        "queue" => "☰",
        "alert" => "⚠",
        "metrics" => "◷",
        "terminal" => ">_",
        "tools" => "🛠",
        "home" => "⌂",
        "logs" => "☷",
        "settings" => "⛭",
        "modules" => "⌘",
        _ => "•",
    }
}
