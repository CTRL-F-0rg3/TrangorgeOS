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
use crate::fs::tui;
use crate::gfx;

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
    // The menu is a TUI, so it needs pixels rather than a character grid — and
    // that is the whole reason it is not a printed list. `gfx::console` is capped
    // at 80 columns, so the 101-column banner could never be drawn through it at
    // any mode; drawn into the framebuffer it is bounded by pixels only.
    if !fit_screen() {
        // No framebuffer to draw a TUI on. Say so and fall back rather than
        // leaving the machine at a menu that renders nothing.
        session::say("no framebuffer available; skipping the display menu\n");
        return gfx::current_resolution();
    }

    let (cur_w, cur_h) = gfx::current_resolution();

    // Start on the mode that is already in effect, so Enter without moving is
    // "keep what I have" rather than an accidental switch.
    let mut sel = MODES
        .iter()
        .position(|(w, h)| *w == cur_w && *h == cur_h)
        .unwrap_or(MODES.len() - 1);

    // Before the menu is drawn and before it can read a key: this changes the
    // mode repeatedly, and a menu drawn halfway through that sequence would be
    // showing a framebuffer that is about to be wiped. It restores the starting
    // mode itself, so the menu still comes up where the user left the machine.
    probe_all_modes();

    draw(&sel);

    // Report the input path once up front, so there is a baseline to compare the
    // heartbeat against even if nothing is ever pressed.
    crate::serial::write_str(&alloc::format!("{}\n", crate::terminal::input_debug()));

    // Spins so far, and the last state reported. Together they let the loop
    // report on *change* rather than on a timer.
    let mut spins: u32 = 0;
    let mut last_dbg: Option<alloc::string::String> = None;

    // Key accounting for the verdict below. `seen` counts keys the menu actually
    // consumed; `sel_moves` counts those that changed the selection. A key that
    // arrives but never moves the selection is a different fault from no key at
    // all, and the two need opposite fixes.
    let mut seen: u32 = 0;
    let mut sel_moves: u32 = 0;
    let sel_at_start = sel;

    loop {
        // Non-blocking, deliberately. The blocking `next_key` halts until a key
        // arrives, so a broken input path made this loop a black box: it simply
        // stopped, with every trace placed after the call unreachable. Polling
        // instead means "nothing arrived" is something the loop can observe and
        // report, rather than something it can only wait forever inside.
        let key = match session::try_key() {
            Some(k) => k,
            None => {
                spins = spins.wrapping_add(1);

                // Periodic rescan, so a keyboard plugged in after boot is
                // found. `scan_ports` was previously called exactly once, during
                // init, which meant a device that appeared later was never
                // seen. Rate-limited because it is a bus-level operation, and
                // `scan_ports` skips ports it has already attached.
                if spins % 65_536 == 0 {
                    crate::drivers::usb::rescan();
                }

                if spins % 4096 == 0 {
                    // Report on *change*, not on a counter. A fixed interval
                    // puts a line on the wire every few hundred milliseconds for
                    // as long as the menu waits, which buries the rest of the boot
                    // log and makes it harder to read than the silence it replaced.
                    // What matters while nothing arrives is that the fields stop
                    // moving - and if they do move, that is exactly when a line is
                    // worth having.
                    let dbg = crate::terminal::input_debug();

                    if last_dbg.as_deref() != Some(dbg.as_str()) {
                        crate::serial::write_str(&alloc::format!(
                            "[menu] idle x{} {dbg}\n",
                            spins / 4096
                        ));
                        last_dbg = Some(dbg);
                    }
                }
                continue;
            }
        };

        // Serial only, never the console: the console is not what is on screen
        // here, and writing to it would paint over the menu this loop is
        // drawing. Its only job is to make "no key ever arrived" distinguishable
        // from "a key arrived and was ignored" when reading a boot log.
        seen += 1;
        crate::serial::write_str(&alloc::format!(
            "[tui] KLUCZ #{seen} {:?} (kod 0x{:x}, kolejka {}, irq1 {})\n",
            key,
            // The keycode as the transport delivered it, not as `Key` reads
            // after decoding: the mapping from one to the other is a place where
            // a key can arrive and still do the wrong thing.
            crate::terminal::keycode_peek().unwrap_or(0),
            crate::terminal::keycode_pending(),
            crate::interrupts::KEYBOARD_HITS.load(core::sync::atomic::Ordering::Relaxed),
        ));

        let sel_before = sel;

        match key {
            // A digit moves the selection; it does not commit it.
            //
            // Committing on the digit is what made this menu look broken. The
            // key did arrive and *was* handled — but `apply` returns from `run`,
            // so the mode set wiped the screen and the menu was simply gone by
            // the time the user looked. The only visible consequence of pressing
            // a key was the menu disappearing, which is indistinguishable from
            // input not working at all. Browsing a list and confirming the
            // choice are separate acts; the menu needs both of them visible.
            Key::Char(c @ '1'..='9') => {
                let i = (c as u8 - b'1') as usize;
                if i < MODES.len() {
                    sel = i;
                }
            }
            Key::Char('j') | Key::Down => sel = (sel + 1) % MODES.len(),
            Key::Char('k') | Key::Up => sel = (sel + MODES.len() - 1) % MODES.len(),

            // Keyboard-backend switching, reachable from the menu because the
            // menu is where a user with a keyboard that "does not work" is
            // standing. Without a command that changes the input mode, a machine
            // whose keyboard is on the wrong backend has no way to say so.
            Key::Char('p') | Key::Char('u') | Key::Char('b') => {
                let want = match key {
                    Key::Char('p') => session::InputBackend::Ps2,
                    Key::Char('u') => session::InputBackend::Usb,
                    _ => session::InputBackend::Both,
                };
                let old = session::set_input_backend(want);

                // On the serial port, not the console: the console is the text
                // under the TUI, and writing there would be painted over by the
                // redraw below anyway.
                crate::serial::write_str(&alloc::format!(
                    "[menu] keyboard {} -> {}\n",
                    session::input_backend_name(),
                    match want {
                        session::InputBackend::Ps2 => "ps/2",
                        session::InputBackend::Usb => "usb",
                        session::InputBackend::Both => "both",
                    }
                ));
                let _ = old;
            }

            Key::Enter => return apply(sel),
            Key::Esc => {
                let (w, h) = gfx::current_resolution();
                session::say(&alloc::format!("staying at {w}x{h}\n"));
                return (w, h);
            }
            // There is no line being typed here, so backspace has nothing to
            // erase. Ignoring it is right; treating it as "go back a row" would
            // be a surprise.
            _ => {}
        }

        // What the menu did about the key, as opposed to what it received. The
        // gap between these two lines is the whole fault: a key that arrives and
        // changes nothing on screen is exactly what "does not react" looks like,
        // and only reporting the redraw tells the two apart.
        if sel != sel_before {
            sel_moves += 1;
        }

        let (cur_w, cur_h) = gfx::current_resolution();
        let (cw, ch) = tui::canvas().map_or((0, 0), |c| (c.width, c.height));
        crate::serial::write_str(&alloc::format!(
            "[tui] sel={sel} -> {}x{} (było {sel_before}, ruchów {sel_moves}) \
             klucze={seen} kolejka={} ekran={cw}x{ch} cur={cur_w}x{cur_h}\n",
            MODES[sel].0,
            MODES[sel].1,
            crate::terminal::keycode_pending(),
        ));
        draw(&sel);

        // The verdict, once the menu has been waiting long enough that silence is
        // no longer explainable by a user who has not touched the keyboard yet.
        //
        // Printed instead of sitting inside the blocking call on purpose: a
        // `next_key` that waits forever can only report "I am waiting", never
        // "nothing is arriving", and those are different faults. `sel_moves`
        // separating from `seen` is what distinguishes a dead key from a
        // mis-mapped one - a key that arrives and does nothing is worse than no
        // key, because the user sees the list react to nothing and assumes the
        // whole thing is stuck.
        if seen == 0 && spins == 262_144 {
            let usb = crate::drivers::usb::class::hid::keyboard_attached();
            let ps2_hits = crate::interrupts::KEYBOARD_HITS
                .load(core::sync::atomic::Ordering::Relaxed);
            crate::serial::write_str(&alloc::format!(
                "[tui] WERDYKT: brak klawiszy. irq1={ps2_hits} usb_klawiatura={usb} \
                 aktywny={}. {}KLIATYSMA: {}\n",
                session::input_backend_name(),
                if usb {
                    "USB jest zalaczona, wiec problem lezy w xHCI, nie w TUI. "
                } else {
                    "USB nie dziala, pada na wylacznik PS/2. "
                },
                "Sprawdz kabel/port."
            ));
        }
    }
}

/// Switch to a mode wide enough for the TUI, and report whether one is available.
///
/// Returns `false` when no framebuffer could be had, which is the one case where
/// there is nothing to draw on.
///
/// The requirement is in pixels: 101 banner columns at one glyph (8 px) is 808
/// pixels, plus a margin. Every offered mode above 1024x768 satisfies that; the
/// smallest that does is chosen so a machine that can be had at 1024x768 is not
/// pushed to 1920x1080 for no reason.
/// Drive every offered mode without a keyboard, and report which one took.
///
/// # Why this exists
///
/// "The resolution menu does not let me change the resolution" has two very
/// different causes that look identical from the outside: the list does not move
/// because no key arrives, or the list moves and Enter is refused. Telling them
/// apart otherwise needs a human to notice which one happened.
///
/// The decision itself does not depend on input at all - it is hardware work
/// indexed by a number - so the menu can make the same decision for itself. This
/// walks every entry, records the outcome, and puts the display back where it
/// started, so a machine that cannot do 1920x1080 finds that out during boot
/// rather than being told "refused" afterwards.
///
/// It runs before the menu is drawn, because each mode set clears the screen
/// and leaving the display mid-sequence would show a blank screen for a second.
pub fn probe_all_modes() {
    let (start_w, start_h) = gfx::current_resolution();

    crate::serial::write_str(&alloc::format!(
        "[tui] autotest trybow: {}, zaczynam od {start_w}x{start_h}\n",
        MODES.len()
    ));

    let mut ok_count = 0u32;

    for (index, &(w, h)) in MODES.iter().enumerate() {
        // Every attempt goes through the same call Enter would make, so a mode
        // reported reachable here is reachable from the menu too. Which mode to
        // pick stays the user's decision; this only establishes which are
        // physically possible.
        // Compared against where the display is *now*, not against where the
        // probe started. The previous mode in the list is normally what is on
        // screen, so comparing to the start skipped a mode that was not
        // actually set - it reported "already there" while the display sat on
        // the previous entry, and left that mode untested.
        let (now_w, now_h) = gfx::current_resolution();
        let same = w == now_w && h == now_h;
        let outcome = if same {
            "juz tak"
        } else if gfx::set_resolution_w_h(w, h) {
            ok_count += 1;
            "OK"
        } else {
            "ODRZUCONE"
        };

        let (cur_w, cur_h) = gfx::current_resolution();
        crate::serial::write_str(&alloc::format!(
            "[tui]   {}. {:>4}x{:<4} -> {outcome:<10} (faktycznie {cur_w}x{cur_h})\n",
            index + 1,
            w,
            h
        ));

        // A mode set goes through the BIOS video path and can leave the 8042 in
        // any state, so the input path is re-asserted between attempts rather
        // than once at the end - the last attempt is as good a place to lose the
        // keyboard as the first.
        crate::terminal::restore_input();
    }

    // Put the display back. Leaving it wherever the last probe landed would make
    // the boot look random, and the user asked for a menu, not a survey.
    if (start_w, start_h) != gfx::current_resolution() {
        if gfx::set_resolution_w_h(start_w, start_h) {
            crate::terminal::restore_input();
            crate::serial::write_str(&alloc::format!(
                "[tui] autotest: przywrocono {start_w}x{start_h}\n"
            ));
        } else {
            // Reported rather than hidden: a display that cannot return to
            // where it started is a fact the user needs.
            let (cur_w, cur_h) = gfx::current_resolution();
            crate::serial::write_str(&alloc::format!(
                "[tui] autotest: NIE DA SIE wrócić do {start_w}x{start_h}, \
                 zostaje {cur_w}x{cur_h}\n"
            ));
        }
    }

    let (cur_w, cur_h) = gfx::current_resolution();
    crate::serial::write_str(&alloc::format!(
        "[tui] autotest: {ok_count} z {} osiągalnych, koniec w {cur_w}x{cur_h}\n",
        MODES.len()
    ));
}

fn fit_screen() -> bool {
    /// One glyph plus the margin, in pixels.
    const NEED_W: u32 = 101 * 8 + 64;
    /// Banner (21 lines) plus a title, a list of nine and a help line, at 16 px.
    const NEED_H: u32 = (21 + 1 + 9 + 1) * 16 + 48;

    if let Some(c) = tui::canvas() {
        if c.width >= NEED_W && c.height >= NEED_H {
            return true;
        }
    }

    let target = MODES
        .iter()
        .find(|&&(w, h)| w >= NEED_W && h >= NEED_H);

    let Some(&(tw, th)) = target else {
        return tui::canvas().is_some();
    };

    if gfx::set_resolution_w_h(tw, th) {
        // A mode set runs through the BIOS video path and can leave the 8042
        // disabled, which would strand the menu with no way to answer it. Put
        // the keyboard back before asking the user for a choice.
        crate::terminal::restore_input();
        session::say(&alloc::format!("display set to {tw}x{th} for the menu\n"));
    }

    tui::canvas().is_some()
}

/// Switch to mode `i` and report whether it took.
fn apply(i: usize) -> (u32, u32) {
    let (w, h) = MODES[i];
    let (cur_w, cur_h) = gfx::current_resolution();

    crate::serial::write_str(&alloc::format!(
        "[tui] apply({i}) {w}x{h}, biezacy {cur_w}x{cur_h}\n"
    ));

    if w == cur_w && h == cur_h {
        // Already there. Saying so beats doing a pointless mode set, which
        // would clear the screen and force a redraw for no reason at all.
        session::say(&alloc::format!("display: keeping {w}x{h}\n"));
        return (w, h);
    }

    if gfx::set_resolution_w_h(w, h) {
        crate::terminal::restore_input();
        session::say(&alloc::format!("display: {w}x{h}\n"));
        crate::serial::write_str(&alloc::format!("[tui] apply({i}) ZAAPLIKOWANO {w}x{h}\n"));
        (w, h)
    } else {
        // `set_resolution_w_h` goes through the Bochs VBE registers, so it can
        // fail on hardware that has none, or for a mode this display does not
        // list. Naming the mode that was refused is the useful part; the machine
        // stays where it was, which is why that is what gets returned.
        session::say(&alloc::format!(
            "display: {w}x{h} refused by this display, staying at {cur_w}x{cur_h}\n"
        ));
        // The reason matters, because "refused" covers three unrelated
        // failures: no Bochs device on the PCI bus, a controller that does not
        // answer, and a framebuffer mapping the memory manager would not hand
        // out. The caller cannot tell them apart and neither can the user.
        let (w_can, h_can) = tui::canvas().map_or((0, 0), |c| (c.width, c.height));
        crate::serial::write_str(&alloc::format!(
            "[tui] apply({i}) ODRZUCONE {w}x{h}. Ekran TUI={w_can}x{h_can}, \
             set_resolution_w_h=false. Sprawdz ./run.sh: potrzebne jest `-vga std` \
             (karta Bochs/VBE). Na prawdziwej karcie GPU ta sciezka nie dziala.\n"
        ));
        (cur_w, cur_h)
    }
}

/// Redraw the whole menu with `sel` highlighted.
///
/// Drawn into the framebuffer, not through the text console. That is what lets
/// the 101-column banner appear at all — the console is capped at 80 columns — and
/// what makes the selection a filled bar rather than a `>` in a text stream.
fn draw(sel: &usize) {
    let Some(c) = tui::canvas() else { return };

    // From here on the TUI is what is on screen, so the turn-over belongs to it
    // and to nothing else.
    tui::activate();

    // The whole screen, not the panel: the mode change wipes the framebuffer and
    // the galaxy background, so the first thing after it is a blank screen.
    tui::fill_rect(&c, 0, 0, c.width as i64, c.height as i64, tui::Color::panel());

    // Layout, in pixels. The banner is at one glyph (8 px) because it is 101
    // characters wide; the list is at two (16 px) because it is nine short lines
    // and worth being readable at a normal viewing distance.
    let margin = 24i64;
    let banner_w = tui::text_width(BANNER, 1);
    let panel_w = banner_w.max(560).min(c.width as i64);
    let panel_x = ((c.width as i64) - panel_w) / 2;
    let panel_y = margin;

    tui::stroke_rect(&c, panel_x, panel_y, panel_w, c.height as i64 - 2 * margin, tui::Color::edge());

    // The banner, verbatim, skipping the blank line the raw string opens with.
    let mut by = panel_y + 12;
    for line in BANNER.lines() {
        if line.trim().is_empty() {
            by += 8;
            continue;
        }
        let x = panel_x + ((panel_w - tui::text_width(line, 1)) / 2).max(0);
        tui::draw_text(&c, x, by, line, tui::Color::edge(), 1);
        by += 8;
    }

    // The mode list, at double scale.
    let (cur_w, cur_h) = gfx::current_resolution();
    let row_h = 16i64;
    let mut ry = by + 20;
    let list_w = 420i64;
    let list_x = panel_x + ((panel_w - list_w) / 2).max(0);

    tui::draw_text(&c, list_x, ry, "  choose a display mode", tui::Color::text(), 2);
    ry += row_h + 8;

    for (i, (w, h)) in MODES.iter().enumerate() {
        let chosen = i == *sel;
        if chosen {
            tui::fill_rect(&c, list_x, ry, list_w, row_h, tui::Color::highlight());
        }

        let text = alloc::format!(" {}. {:>4}x{:<4}", i + 1, w, h);
        tui::draw_text(
            &c,
            list_x + 8,
            ry,
            &text,
            if chosen { tui::Color::text_on_highlight() } else { tui::Color::text() },
            2,
        );

        if *w == cur_w && *h == cur_h {
            tui::draw_text(&c, list_x + 260, ry, "(current)", tui::Color::dim(), 2);
        }

        ry += row_h;
    }

    ry += 12;
    tui::draw_text(
        &c,
        list_x,
        ry,
        "Up/Down or j/k to move   1-9 to select   Enter confirm   Esc keep current",
        tui::Color::dim(),
        1,
    );

    // Keyboard-backend line, and the reason it is on screen: this is a menu
    // that only responds to a keyboard, so a user whose keyboard is on the
    // other backend has no way to recover except by being told the shortcut
    // exists. Rendered from the live setting rather than a literal so it cannot
    // drift from what the driver is actually doing.
    ry += 16;
    tui::draw_text(
        &c,
        list_x,
        ry,
        &alloc::format!(
            "keyboard: {}   (p) ps/2   (u) usb   (b) both",
            session::input_backend_name()
        ),
        tui::Color::dim(),
        1,
    );
}


