use gpui::{svg, Rgba, Svg};

pub const SPLASH_BG_TOP: Rgba = Rgba {
    r: 0.060,
    g: 0.055,
    b: 0.580,
    a: 1.0,
};
pub const SPLASH_BG_MID: Rgba = Rgba {
    r: 0.205,
    g: 0.105,
    b: 0.760,
    a: 1.0,
};
pub const SPLASH_BG_HORIZON: Rgba = Rgba {
    r: 0.790,
    g: 0.240,
    b: 0.760,
    a: 1.0,
};
pub const SPLASH_BG_BOTTOM: Rgba = Rgba {
    r: 1.000,
    g: 0.480,
    b: 0.560,
    a: 1.0,
};

pub const SPLASH_HORIZON_PINK: Rgba = Rgba {
    r: 1.000,
    g: 0.250,
    b: 0.670,
    a: 1.0,
};
pub const SPLASH_HORIZON_GOLD: Rgba = Rgba {
    r: 1.000,
    g: 0.690,
    b: 0.250,
    a: 1.0,
};

pub const SPLASH_HILL_BACK: Rgba = Rgba {
    r: 0.430,
    g: 0.150,
    b: 0.900,
    a: 0.62,
};
pub const SPLASH_HILL_LEFT: Rgba = Rgba {
    r: 0.500,
    g: 0.180,
    b: 0.920,
    a: 0.78,
};
pub const SPLASH_HILL_RIGHT: Rgba = Rgba {
    r: 0.420,
    g: 0.135,
    b: 0.835,
    a: 0.76,
};
pub const SPLASH_HILL_FRONT: Rgba = Rgba {
    r: 0.045,
    g: 0.040,
    b: 0.430,
    a: 0.82,
};

pub const SPLASH_ARCH_CORE: Rgba = Rgba {
    r: 0.930,
    g: 0.860,
    b: 1.000,
    a: 1.0,
};
pub const SPLASH_ARCH_GLOW: Rgba = Rgba {
    r: 0.765,
    g: 0.610,
    b: 1.000,
    a: 1.0,
};

pub const SPLASH_SUN_LAYERS: [(f32, f32, Rgba); 5] = [
    (
        4.2,
        0.055,
        Rgba {
            r: 1.000,
            g: 0.300,
            b: 0.620,
            a: 1.0,
        },
    ),
    (
        3.2,
        0.115,
        Rgba {
            r: 1.000,
            g: 0.500,
            b: 0.280,
            a: 1.0,
        },
    ),
    (
        2.2,
        0.210,
        Rgba {
            r: 1.000,
            g: 0.690,
            b: 0.300,
            a: 1.0,
        },
    ),
    (
        1.45,
        0.440,
        Rgba {
            r: 1.000,
            g: 0.830,
            b: 0.520,
            a: 1.0,
        },
    ),
    (
        1.0,
        1.000,
        Rgba {
            r: 1.000,
            g: 0.950,
            b: 0.770,
            a: 1.0,
        },
    ),
];

pub const SPLASH_STAR: Rgba = Rgba {
    r: 1.0,
    g: 1.0,
    b: 1.0,
    a: 1.0,
};

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

pub fn module_row_bg(is_dark: bool) -> Rgba {
    if is_dark {
        Rgba {
            r: 0.120,
            g: 0.150,
            b: 0.190,
            a: 1.0,
        }
    } else {
        Rgba {
            r: 1.000,
            g: 1.000,
            b: 1.000,
            a: 0.98,
        }
    }
}

pub fn module_row_stroke(is_dark: bool) -> Rgba {
    if is_dark {
        Rgba {
            r: 0.260,
            g: 0.315,
            b: 0.390,
            a: 1.0,
        }
    } else {
        Rgba {
            r: 0.860,
            g: 0.905,
            b: 0.965,
            a: 1.0,
        }
    }
}

pub fn module_title_text(is_dark: bool) -> Rgba {
    if is_dark {
        Rgba {
            r: 0.940,
            g: 0.965,
            b: 1.000,
            a: 1.0,
        }
    } else {
        Rgba {
            r: 0.090,
            g: 0.145,
            b: 0.230,
            a: 1.0,
        }
    }
}

pub fn module_description_text(is_dark: bool) -> Rgba {
    if is_dark {
        Rgba {
            r: 0.700,
            g: 0.760,
            b: 0.840,
            a: 1.0,
        }
    } else {
        Rgba {
            r: 0.320,
            g: 0.390,
            b: 0.500,
            a: 1.0,
        }
    }
}

pub fn module_meta_text(is_dark: bool) -> Rgba {
    if is_dark {
        Rgba {
            r: 0.775,
            g: 0.835,
            b: 0.930,
            a: 1.0,
        }
    } else {
        Rgba {
            r: 0.250,
            g: 0.390,
            b: 0.660,
            a: 1.0,
        }
    }
}

pub fn module_state_enabled_bg(is_dark: bool) -> Rgba {
    if is_dark {
        Rgba {
            r: 0.070,
            g: 0.350,
            b: 0.255,
            a: 0.38,
        }
    } else {
        Rgba {
            r: 0.820,
            g: 0.970,
            b: 0.895,
            a: 1.0,
        }
    }
}

pub fn module_state_enabled_text(is_dark: bool) -> Rgba {
    if is_dark {
        Rgba {
            r: 0.635,
            g: 0.955,
            b: 0.825,
            a: 1.0,
        }
    } else {
        Rgba {
            r: 0.060,
            g: 0.460,
            b: 0.340,
            a: 1.0,
        }
    }
}

pub fn module_state_disabled_bg(is_dark: bool) -> Rgba {
    if is_dark {
        Rgba {
            r: 0.420,
            g: 0.180,
            b: 0.190,
            a: 0.35,
        }
    } else {
        Rgba {
            r: 0.995,
            g: 0.880,
            b: 0.895,
            a: 1.0,
        }
    }
}

pub fn module_state_disabled_text(is_dark: bool) -> Rgba {
    if is_dark {
        Rgba {
            r: 0.985,
            g: 0.760,
            b: 0.785,
            a: 1.0,
        }
    } else {
        Rgba {
            r: 0.700,
            g: 0.180,
            b: 0.250,
            a: 1.0,
        }
    }
}

pub fn module_button_enable_bg(is_dark: bool) -> Rgba {
    if is_dark {
        Rgba {
            r: 0.120,
            g: 0.465,
            b: 0.335,
            a: 1.0,
        }
    } else {
        Rgba {
            r: 0.130,
            g: 0.610,
            b: 0.435,
            a: 1.0,
        }
    }
}

pub fn module_button_enable_text(is_dark: bool) -> Rgba {
    if is_dark {
        Rgba {
            r: 0.905,
            g: 1.000,
            b: 0.960,
            a: 1.0,
        }
    } else {
        Rgba {
            r: 0.950,
            g: 1.000,
            b: 0.975,
            a: 1.0,
        }
    }
}

pub fn module_button_disable_bg(is_dark: bool) -> Rgba {
    if is_dark {
        Rgba {
            r: 0.620,
            g: 0.250,
            b: 0.285,
            a: 1.0,
        }
    } else {
        Rgba {
            r: 0.870,
            g: 0.285,
            b: 0.365,
            a: 1.0,
        }
    }
}

pub fn module_button_disable_text(is_dark: bool) -> Rgba {
    if is_dark {
        Rgba {
            r: 1.000,
            g: 0.925,
            b: 0.940,
            a: 1.0,
        }
    } else {
        Rgba {
            r: 1.000,
            g: 0.955,
            b: 0.965,
            a: 1.0,
        }
    }
}

/// Neutral compact pill in the main top bar (matches cwd / small actions).
pub fn top_bar_pill_bg(is_dark: bool) -> Rgba {
    if is_dark {
        Rgba {
            r: 0.122,
            g: 0.161,
            b: 0.216,
            a: 1.0,
        }
    } else {
        Rgba {
            r: 0.953,
            g: 0.957,
            b: 0.961,
            a: 1.0,
        }
    }
}

pub fn top_bar_pill_text(is_dark: bool) -> Rgba {
    if is_dark {
        Rgba {
            r: 0.820,
            g: 0.847,
            b: 0.859,
            a: 1.0,
        }
    } else {
        Rgba {
            r: 0.294,
            g: 0.337,
            b: 0.388,
            a: 1.0,
        }
    }
}

pub fn top_bar_pill_hover_bg(is_dark: bool) -> Rgba {
    if is_dark {
        Rgba {
            r: 0.165,
            g: 0.212,
            b: 0.278,
            a: 1.0,
        }
    } else {
        Rgba {
            r: 0.922,
            g: 0.929,
            b: 0.941,
            a: 1.0,
        }
    }
}

/// Selected top-bar nav pill (e.g. Logs when that page is active).
pub fn top_bar_pill_active_bg(is_dark: bool) -> Rgba {
    if is_dark {
        Rgba {
            r: 0.122,
            g: 0.165,
            b: 0.243,
            a: 1.0,
        }
    } else {
        Rgba {
            r: 0.882,
            g: 0.906,
            b: 1.000,
            a: 1.0,
        }
    }
}

pub fn top_bar_pill_active_text(is_dark: bool) -> Rgba {
    if is_dark {
        Rgba {
            r: 0.576,
            g: 0.773,
            b: 0.992,
            a: 1.0,
        }
    } else {
        Rgba {
            r: 0.114,
            g: 0.306,
            b: 0.847,
            a: 1.0,
        }
    }
}

pub fn top_bar_pill_active_hover_bg(is_dark: bool) -> Rgba {
    if is_dark {
        Rgba {
            r: 0.145,
            g: 0.196,
            b: 0.282,
            a: 1.0,
        }
    } else {
        Rgba {
            r: 0.855,
            g: 0.878,
            b: 0.992,
            a: 1.0,
        }
    }
}
