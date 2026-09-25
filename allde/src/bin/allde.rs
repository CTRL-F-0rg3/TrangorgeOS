//! `allde` — a scripted session of the all-de desktop environment.
//!
//! Spawns several apps (multiple shell terminals + a status bar + an info
//! panel), types into the terminals, moves a window in the tiling, then renders
//! the desktop frame to `allde_frame.ppm`.

use allde::{Allde, AppKind, Key};

fn write_ppm(path: &str, fb: &[u32], w: u32, h: u32) -> std::io::Result<()> {
    use std::io::Write;
    let mut f = std::fs::File::create(path)?;
    write!(f, "P6\n{} {}\n255\n", w, h)?;
    let mut data = Vec::with_capacity((w * h * 3) as usize);
    for px in fb {
        data.push(((px >> 16) & 0xFF) as u8);
        data.push(((px >> 8) & 0xFF) as u8);
        data.push((px & 0xFF) as u8);
    }
    f.write_all(&data)?;
    Ok(())
}

fn type_line(de: &mut Allde, line: &str) {
    for c in line.chars() {
        de.type_char(c);
    }
    de.type_char('\n');
}

fn main() {
    let (w, h) = (800u32, 600u32);
    let mut fb = vec![0u32; (w * h) as usize];
    let mut de = Allde::new(w, h);

    // 1. Spawn several independent processes / windows.
    let t1 = de.spawn_shell("terminal 1");
    let t2 = de.spawn_shell("terminal 2");
    de.spawn_shell("terminal 3");
    de.spawn_app(AppKind::StatusBar, "status");
    de.spawn_app(AppKind::Info, "info");

    // 2. Type into the first terminal (echo + uname).
    de.wm.focus(t1);
    type_line(&mut de, "echo hello from allde");
    type_line(&mut de, "uname");

    // 3. Type into the second terminal, then spawn a 4th terminal via `new`.
    de.wm.focus(t2);
    type_line(&mut de, "list");
    type_line(&mut de, "new");

    // 4. Move the first terminal one column to the right (tiling reorder).
    de.wm.focus(t1);
    de.handle_key(Key::Right);

    // 5. Render the desktop.
    de.render(&mut fb);
    write_ppm("allde_frame.ppm", &fb, w, h).expect("write ppm");
    println!(
        "[allde] rendered allde_frame.ppm ({}x{}, {} processes)",
        w,
        h,
        de.procs.count()
    );
}
