//! Serialize vt100 screen rows to ANSI strings for colored shell scrollback.

use std::fmt::Write;

use vt100::Color;

fn push_fg(out: &mut String, c: Color) {
    match c {
        Color::Default => out.push_str("\x1b[39m"),
        Color::Idx(i) => {
            let _ = write!(out, "\x1b[38;5;{i}m");
        }
        Color::Rgb(r, g, b) => {
            let _ = write!(out, "\x1b[38;2;{r};{g};{b}m");
        }
    }
}

fn push_bg(out: &mut String, c: Color) {
    match c {
        Color::Default => out.push_str("\x1b[49m"),
        Color::Idx(i) => {
            let _ = write!(out, "\x1b[48;5;{i}m");
        }
        Color::Rgb(r, g, b) => {
            let _ = write!(out, "\x1b[48;2;{r};{g};{b}m");
        }
    }
}

fn strip_escapes_for_blank_check(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut it = s.chars().peekable();
    while let Some(ch) = it.next() {
        if ch == '\x1b' {
            match it.peek().copied() {
                Some('[') => {
                    it.next();
                    while let Some(c) = it.next() {
                        if ('\x40'..='\x7e').contains(&c) {
                            break;
                        }
                    }
                }
                Some(']') => {
                    it.next();
                    while let Some(c) = it.next() {
                        if c == '\x07' {
                            break;
                        }
                        if c == '\x1b' && it.peek() == Some(&'\\') {
                            let _ = it.next();
                            break;
                        }
                    }
                }
                _ => {}
            }
            continue;
        }
        out.push(ch);
    }
    out
}

/// Paint one row as SGR runs for [`super::ansi_line::shell_history_line`].
pub(crate) fn vt100_row_for_shell_history(
    screen: &vt100::Screen,
    row: u16,
    cols: u16,
) -> Option<String> {
    let mut out = String::new();
    let mut prev: Option<(Color, Color, bool)> = None;

    for col in 0..cols {
        let cell = screen.cell(row, col);
        let (nf, nb, nbold, text) = match cell {
            Some(cell) => (
                cell.fgcolor(),
                cell.bgcolor(),
                cell.bold(),
                cell.contents(),
            ),
            None => (
                Color::Default,
                Color::Default,
                false,
                " ".to_string(),
            ),
        };
        let key = (nf, nb, nbold);
        if prev.as_ref() != Some(&key) {
            out.push_str("\x1b[0m");
            push_fg(&mut out, nf);
            push_bg(&mut out, nb);
            if nbold {
                out.push_str("\x1b[1m");
            }
            prev = Some(key);
        }
        if text.is_empty() {
            out.push(' ');
        } else {
            out.push_str(&text);
        }
    }
    out.push_str("\x1b[0m");

    let trimmed = if let Some(pos) = out.rfind("\x1b[0m") {
        let body = &out[..pos];
        let tail = &out[pos..];
        format!("{}{}", body.trim_end_matches(' '), tail)
    } else {
        out.trim_end_matches(' ').to_string()
    };

    if strip_escapes_for_blank_check(&trimmed)
        .chars()
        .all(|c| c.is_whitespace())
    {
        None
    } else {
        Some(trimmed)
    }
}
