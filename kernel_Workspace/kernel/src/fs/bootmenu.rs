//! The pre-login resolution menu.
//!
//! # Why this is here
//!
//! Picking a mode before the login prompt is a Redox-style boot chooser, and it
//! exists here for a concrete reason rather than for looks: this kernel ships
//! with a 320x200 console, and a 1920x1080 mode set *after* login looks like a
//! mistake, because the text you were reading is already laid out for the old
//! mode. Choosing first means the login prompt is drawn once, at the size it
//! will stay.
//!
//! # Input
//!
//! Two key paths reach this menu and they are not equally capable:
//!
//! * The PS/2 keyboard produces *keycodes*, including arrows and Escape.
//! * A USB HID keyboard produces *characters only* — `key_to_ascii` in the HID
//!   driver maps no arrow key, so arrows are simply not a thing it can report.
//!
//! So the menu is built around digits, which both can send, and treats the
//! arrows as an extra that happens to work on PS/2 hardware. A menu that only
//! worked with arrows would be unusable on a USB keyboard, which is the more
//! common of the two.

use crate::fs::session::{self, Key};
use crate::gfx;
use crate::vga_buffer::Color;

/// The modes offered, in the order they are listed.
///
/// A deliberate spread: low modes for a machine with no framebuffer to speak of,
/// the two common laptop panels, and the 1080p default. Not every one of these
/// is settable on every machine — see the note on `set_resolution_w_h` below —
/// so the menu reports a refusal rather than pretending to switch.
pub const MODES: &[(u32, u32)] = &[
    (320, 200),
    (640, 480),
    (800, 600),
    (1024, 768),
    (1152, 864),
    (1280, 720),
    (1280, 1024),
    (1600, 1200),
    (1920, 1080),
];

/// The banner, drawn with the menu so the screen is not a bare list.
///
/// Verbatim from `baner.txt` at the repository root, which is the source of
/// truth for this artwork — regenerate it there and copy the result, rather
/// than editing the art in here where nobody looks for it.
///
/// A raw string with a `#` fence: the art contains both backslashes and
/// backticks, so `r"..."` would terminate on the first backtick.
const BANNER: &str = r#"
>>=====================================================================================================<<
||                                                                                                     ||
||                                                                                                     ||
||                                                                                                     ||
||                                                                                                     ||
||                                                                                                     ||
||      ______                                                                 _____   ____            ||
||     /\__  _\                                                               /\  __`\/\  _`\          ||
||     \/_/\ \/ _ __    __      ___      __     ___   _ __    __      __      \ \ \/\ \ \,\L\_\        ||
||        \ \ \/\`'__\/'__`\  /' _ `\  /'_ `\  / __`\/\`'__\/'_ `\  /'__`\     \ \ \ \ \/_\__ \        ||
||         \ \ \ \ \//\ \L\.\_/\ \/\ \/\ \L\ \/\ \L\ \ \ \//\ \L\ \/\  __/      \ \ \_\ \/\ \L\ \      ||
||          \ \_\ \_\\ \__/.\_\ \_\ \_\ \____ \ \____/\ \_\\ \____ \ \____\      \ \_____\ `\____\     ||
||           \/_/\/_/ \/__/\/_/\/_/\/_/\/___L\ \/___/  \/_/ \/___L\ \/____/       \/_____/\/_____/     ||
||                                       /\____/              /\____/                                  ||
||                                       \_/__/               \_/__/                                   ||
||                                                                                                     ||
||                                                                                                     ||
||                                                                                                     ||
||                                                                                                     ||
||                                                                                                     ||
>>=====================================================================================================<<
"#;

/// The narrowest console the menu can be drawn on, in text cells.
///
/// The banner is 101 columns wide, and the whole screen it is drawn on is the
/// banner plus nine mode rows plus the help lines. Both have to fit, or the art
/// wraps and the menu stops being a menu.
///
/// This is a VGA text-cell budget rather than a pixel one, because the banner is
/// text: what matters is how many character cells the current mode provides, not
/// how many pixels it has.
const NEED_COLS: usize = 101;
const NEED_ROWS: usize = 21 + 1 + 9 + 3;

/// Draw the menu and apply whatever the user picks.
///
/// Returns the mode that was actually applied, which is the one already in use
/// if the user pressed Escape or every candidate mode was refused.
pub fn run() -> (u32, u32) {
    // The art is 101 columns wide and the whole screen it is drawn on needs about
    // 35 rows. The kernel's default console is 320x200, which is 80x25 — smaller
    // than both — so drawing into it would wrap every line of the banner and
    // scatter the mode list below the scroll. Rather than mangle the artwork or
    // drop it, the mode is raised to one that can hold it.
    fit_screen();

    let (cur_w, cur_h) = gfx::current_resolution();

    // Start on the mode that is already in effect, so Enter without moving is
    // "keep what I have" rather than an accidental switch to 320x200.
    let mut sel = MODES
        .iter()
        .position(|(w, h)| *w == cur_w && *h == cur_h)
        .unwrap_or(MODES.len() - 1);

    draw(&sel);

    loop {
        let key = match session::next_key() {
            Some(k) => k,
            None => continue,
        };

        match key {
            // Digits pick a row directly. A digit also confirms, because the
            // point of the menu is to set a mode, not to browse one: making the
            // user press a digit and then Enter is a step for its own sake.
            Key::Char(c @ '1'..='9') => {
                let i = (c as u8 - b'1') as usize;
                if i < MODES.len() {
                    return apply(i);
                }
            }
            Key::Char('j') => sel = (sel + 1) % MODES.len(),
            Key::Char('k') => sel = (sel + MODES.len() - 1) % MODES.len(),
            Key::Up => sel = (sel + MODES.len() - 1) % MODES.len(),
            Key::Down => sel = (sel + 1) % MODES.len(),
            Key::Enter => return apply(sel),
            Key::Esc => {
                let (w, h) = gfx::current_resolution();
                session::say(&alloc::format!("\n  staying at {w}x{h}\n\n"));
                return (w, h);
            }
            // There is no line being typed here, so backspace has nothing to
            // erase. Ignoring it is right; treating it as "go back a row" would
            // be a surprise.
            _ => {}
        }

        draw(&sel);
    }
}

/// Switch to a mode large enough to hold the banner and the mode list.
///
/// The smallest offered mode with room for both, tried in order. Each candidate
/// is a plain mode from [`MODES`], so this is the same code path the menu itself
/// uses — no separate, untested way of changing the display.
///
/// Failing to find one is not fatal: the menu still runs, the banner wraps, and
/// the user can still pick a mode with the digit keys. That is strictly better
/// than refusing to show the menu at all.
fn fit_screen() {
    let (w, h) = gfx::current_resolution();

    // The console reports its own grid, which is the number that actually
    // decides whether a 101-column line fits — pixels alone would be a guess
    // about the font.
    if gfx::console::cols() >= NEED_COLS && gfx::console::rows() >= NEED_ROWS {
        return;
    }

    for (cw, ch) in MODES {
        if !gfx::set_resolution_w_h(cw, ch) {
            continue;
        }
        if gfx::console::cols() >= NEED_COLS && gfx::console::rows() >= NEED_ROWS {
            session::say(&alloc::format!(
                "display raised to {cw}x{ch} to fit the menu\n\n"
            ));
            return;
        }
    }

    // Nothing was wide enough, so the mode the last attempt left behind has to
    // be put back: a failed attempt still changed it.
    let _ = gfx::set_resolution_w_h(w, h);
    session::say(&alloc::format!(
        "no offered mode fits the {NEED_COLS}-column menu; drawing it wrapped\n\n"
    ));
}

/// Switch to mode `i` and report whether it took.
fn apply(i: usize) -> (u32, u32) {
    let (w, h) = MODES[i];
    let (cur_w, cur_h) = gfx::current_resolution();

    session::say("\n");
    if w == cur_w && h == cur_h {
        // Already there. Saying so beats doing a pointless mode set, which
        // would clear the screen and force a redraw for no reason at all.
        session::say(&alloc::format!("  keeping {w}x{h}\n\n"));
        return (w, h);
    }

    if gfx::set_resolution_w_h(w, h) {
        session::say(&alloc::format!("  resolution set to {w}x{h}\n\n"));
        (w, h)
    } else {
        // `set_resolution_w_h` goes through the Bochs VBE registers, so it can
        // fail on hardware that has none, or for a mode this display does not
        // list. Naming the mode that was refused is the useful part; the machine
        // stays where it was, which is why that is what gets returned.
        session::say(&alloc::format!(
            "  {w}x{h} refused by this display; staying at {cur_w}x{cur_h}\n\n"
        ));
        (cur_w, cur_h)
    }
}

/// Redraw the menu with `sel` highlighted.
fn draw(sel: &usize) {
    // Cleared rather than appended to: the list is redrawn on every keypress,
    // and scrolling it down the screen would fill the console with copies.
    //
    // `clear_screen` lives on the writer rather than on `gfx::console` because
    // it is the writer that owns the text cursor: clearing a framebuffer and
    // forgetting the cursor would leave the next character mid-row.
    crate::vga_buffer::WRITER.lock().clear_screen();
    session::say(BANNER);
    session::say("\n");

    let (cur_w, cur_h) = gfx::current_resolution();

    for (i, (w, h)) in MODES.iter().enumerate() {
        let current = if *w == cur_w && *h == cur_h { "  (current)" } else { "" };
        let mark = if i == *sel { '>' } else { ' ' };
        // The number is always shown, not only on the selected row: the arrows
        // are a PS/2-only affordance, so on a USB keyboard the digits are the
        // only way to move and they have to be visible on every row.
        let line = alloc::format!(" {mark} {}. {:>4}x{:<4}{current}\n", i + 1, w, h);
        if i == *sel {
            session::say_coloured(&line, Color::LightCyan);
        } else {
            session::say(&line);
        }
    }

    session::say("\n  Up/Down or j/k to move, 1-9 to pick and apply,\n");
    session::say("  Enter to confirm, Esc to keep the current mode.\n\n");
    gfx::refresh();
}

