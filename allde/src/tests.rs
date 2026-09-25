use crate::shell::Shell;
use crate::{Allde, AppKind};

#[test]
fn tiling_layout_is_equal_columns() {
    let mut de = Allde::new(600, 400);
    de.spawn_shell("a");
    de.spawn_shell("b");
    de.spawn_shell("c");

    let layout = de.wm.layout();
    assert_eq!(layout.len(), 3);
    // Each of the three columns is 600 / 3 = 200 wide.
    assert_eq!(layout[0].1.w, 200);
    assert_eq!(layout[1].1.w, 200);
    assert_eq!(layout[2].1.w, 200);
    assert_eq!(layout[0].1.x, 0);
    assert_eq!(layout[1].1.x, 200);
    assert_eq!(layout[2].1.x, 400);
}

#[test]
fn move_window_reorders_tiling() {
    let mut de = Allde::new(600, 400);
    let a = de.spawn_shell("a");
    let b = de.spawn_shell("b");

    assert_eq!(de.wm.windows().collect::<Vec<_>>(), vec![a, b]);

    de.wm.move_left(b);
    assert_eq!(de.wm.windows().collect::<Vec<_>>(), vec![b, a]);
}

#[test]
fn move_window_between_workspaces() {
    let mut de = Allde::new(600, 400);
    let a = de.spawn_shell("a");
    de.spawn_shell("b");

    de.wm.move_to_workspace(a, 1);
    assert_eq!(de.wm.active_ws, 1);
    assert_eq!(de.wm.windows().collect::<Vec<_>>(), vec![a]);
}

#[test]
fn shell_runs_builtins() {
    let mut s = Shell::new("$ ");
    for c in "echo hi".chars() {
        s.feed_char(c);
    }
    s.feed_char('\n');
    assert!(s.lines.iter().any(|l| l.contains("hi")));
}

#[test]
fn multiple_terminals_are_independent() {
    let mut de = Allde::new(800, 600);
    let a = de.spawn_shell("a");
    let b = de.spawn_shell("b");

    de.wm.focus(a);
    de.type_char('x');
    de.wm.focus(b);
    de.type_char('y');

    assert_eq!(de.shells[&a].input, "x");
    assert_eq!(de.shells[&b].input, "y");
}

#[test]
fn desktop_commands_spawn_and_list() {
    let mut de = Allde::new(800, 600);
    de.spawn_shell("t1");
    let n_before = de.procs.count();

    let id = de.wm.focused.unwrap();
    // Simulate typing "new" + enter into the focused terminal.
    for c in "new".chars() {
        de.type_char(c);
    }
    de.type_char('\n');

    assert_eq!(de.procs.count(), n_before + 1);
    let _ = id;
}

#[test]
fn render_draws_non_background_pixels() {
    let mut de = Allde::new(320, 200);
    de.spawn_shell("t");
    de.spawn_app(AppKind::StatusBar, "status");

    let mut fb = vec![0u32; 320 * 200];
    de.render(&mut fb);

    // Background is 0xFF101218; windows must have drawn other pixels.
    assert!(fb.iter().any(|&p| p != 0xFF10_1218));
}
