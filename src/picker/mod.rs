pub mod state;

use crate::ui::{Palette, palette};
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    execute, queue,
    terminal::{self, Clear, ClearType},
};
use state::PickerState;
use std::io::Write;

fn fallback(prompt: &str, items: &[String]) -> Option<usize> {
    let p = palette();
    let mut stderr = std::io::stderr();
    let _ = writeln!(stderr, "\n{}{}{}", p.bold, prompt, p.reset);
    for (i, item) in items.iter().enumerate() {
        let _ = writeln!(stderr, "  {:>2}) {}", i + 1, item);
    }
    let _ = write!(stderr, "\nChoice (1-{}): ", items.len());
    let _ = stderr.flush();

    let line = crate::ui::read_tty_line().unwrap_or_default();
    match line.trim().parse::<usize>() {
        Ok(n) if n >= 1 && n <= items.len() => Some(n - 1),
        _ => Some(0),
    }
}

fn abort() -> ! {
    let _ = terminal::disable_raw_mode();
    let mut out = std::io::stderr();
    let _ = execute!(out, cursor::Show);
    std::process::exit(1);
}

pub fn select(prompt: &str, items: &[String]) -> Option<usize> {
    if items.is_empty() {
        return None;
    }
    if terminal::enable_raw_mode().is_err() {
        return fallback(prompt, items);
    }
    let p = palette();
    let mut s = state::new(items.to_vec());
    let mut out = std::io::BufWriter::new(std::io::stderr());
    let _ = execute!(out, cursor::Hide);
    // header
    let _ = write!(out, "\n  {}{}{}\r\n\n", p.accent_bold, prompt, p.reset);
    // track the line count of the last draw so each redraw can move the cursor
    // back to the top of the list (MoveUp) and clear downward — relative cursor
    // movement is honored everywhere, unlike Save/RestorePosition.
    let mut last_n = draw(&mut out, &s, &p);

    loop {
        match event::read() {
            Ok(Event::Key(KeyEvent {
                code,
                modifiers,
                kind: KeyEventKind::Press,
                ..
            })) => {
                if modifiers.contains(KeyModifiers::CONTROL) && code == KeyCode::Char('c') {
                    abort();
                }
                match code {
                    KeyCode::Up | KeyCode::Char('k') => s.move_up(),
                    KeyCode::Down | KeyCode::Char('j') => s.move_down(),
                    KeyCode::Backspace => s.backspace(),
                    KeyCode::Esc => s.clear_filter(),
                    KeyCode::Enter => break,
                    KeyCode::Char('q') => {
                        if s.filter.is_empty() {
                            abort();
                        } else {
                            s.push_filter('q');
                        }
                    }
                    KeyCode::Char(c) => s.push_filter(c),
                    _ => {}
                }
                // move back to the top of the list, wipe old list, draw fresh
                if last_n > 0 {
                    let _ = queue!(out, cursor::MoveUp(last_n as u16));
                }
                let _ = queue!(out, Clear(ClearType::FromCursorDown));
                last_n = draw(&mut out, &s, &p);
            }
            Ok(_) => {}
            Err(_) => {
                // event source failed (e.g. mio couldn't register the tty fd);
                // clean up the picker UI and fall back to line-buffered input
                if last_n > 0 {
                    let _ = queue!(out, cursor::MoveUp(last_n as u16));
                }
                let _ = execute!(out, Clear(ClearType::FromCursorDown), cursor::Show);
                let _ = terminal::disable_raw_mode();
                return fallback(prompt, items);
            }
        }
    }

    let chosen = s.selected_original();
    // collapse to the single selected line
    if last_n > 0 {
        let _ = queue!(out, cursor::MoveUp(last_n as u16));
    }
    let _ = queue!(out, Clear(ClearType::FromCursorDown));
    if let Some(orig) = chosen {
        let _ = write!(
            out,
            "  {}❯{} {}{}{}\r\n",
            p.accent_bold, p.reset, p.bold, items[orig], p.reset
        );
    }
    let _ = execute!(out, cursor::Show);
    let _ = terminal::disable_raw_mode();
    chosen
}

// Draw the search line, scroll hints, and the visible window. Returns line count.
fn draw<W: Write>(out: &mut W, s: &PickerState, p: &Palette) -> usize {
    let mut n = 0;
    // search line
    if s.filter.is_empty() {
        let _ = write!(out, "  {}/ type to filter{}", p.muted, p.reset);
    } else {
        let _ = write!(out, "  {}/ {}{}", p.accent, s.filter, p.reset);
    }
    let _ = write!(out, "\x1b[K\r\n");
    n += 1;

    let nv = s.visible.len();
    let end = (s.top + s.view).min(nv);

    if s.top > 0 {
        let _ = write!(out, "  {}↑ {} more{}\x1b[K\r\n", p.muted, s.top, p.reset);
        n += 1;
    }

    if nv == 0 {
        let _ = write!(out, "  {}no matches{}\x1b[K\r\n", p.muted, p.reset);
        n += 1;
    } else {
        for i in s.top..end {
            let orig = s.visible[i];
            if i == s.cur {
                let _ = write!(
                    out,
                    "  {}❯{} {}{}\x1b[K\r\n",
                    p.accent_bold, p.reset, s.items[orig], p.reset
                );
            } else {
                let _ = write!(
                    out,
                    "  {}·{} {}{}{}\x1b[K\r\n",
                    p.muted, p.reset, p.muted, s.items[orig], p.reset
                );
            }
            n += 1;
        }
    }

    let remaining = nv.saturating_sub(end);
    if remaining > 0 {
        let _ = write!(
            out,
            "  {}↓ {} more{}\x1b[K\r\n",
            p.muted, remaining, p.reset
        );
    } else {
        let _ = write!(out, "\x1b[K\r\n");
    }
    n += 1;
    let _ = out.flush();
    n
}
