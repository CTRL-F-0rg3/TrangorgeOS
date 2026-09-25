//! `demouserspace` — a sample graphical window for the TrangorgeOS userspace.
//!
//! Demonstrates the nested userspace end-to-end:
//!   1. a session spawns an app with *derived* (subset) capabilities,
//!   2. the app draws a window with a terminal through the compositor,
//!   3. the framebuffer is written to `demo_window.ppm` (on the real system the
//!      kernel-provided buffer is presented instead).

use uspace::caps::{CapEntry, CapId, ObjectType, Process, Rights, Session};
use uspace::gfx::draw_demo_window;

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

fn main() {
    // 1. Nested permission model: a session spawns an app with derived caps.
    let mut session = Session::new();
    let mut root = Process::new(0, None);
    root.grant(CapId(1), CapEntry::new(0, ObjectType::Device, Rights::CALL));

    let app = session.spawn_derived(
        &root,
        &[(CapId(1), CapEntry::new(0, ObjectType::Device, Rights::CALL))],
    );
    println!(
        "[demouserspace] session spawned app pid={:?} with derived caps",
        app.as_ref().map(|p| p.pid)
    );

    // 2. Draw a window (with a terminal) into a kernel-provided framebuffer.
    let (w, h) = (800u32, 600u32);
    let mut fb = vec![0u32; (w * h) as usize];
    draw_demo_window(&mut fb, w, h);

    // 3. Present: on the host we dump a PPM; on the OS the buffer is the
    //    kernel framebuffer (see uspace::kernel::KernelClient).
    write_ppm("demo_window.ppm", &fb, w, h).expect("failed to write demo_window.ppm");
    println!("[demouserspace] wrote demo_window.ppm ({}x{})", w, h);
}
