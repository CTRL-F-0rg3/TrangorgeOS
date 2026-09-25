# allde — niri-like tiling desktop environment for TrangorgeOS userspace

A full desktop environment for the userspace, modelled after the *niri* Wayland
compositor: windows are **tiled automatically** into columns, can be **moved
between columns and workspaces**, and are **focused by keyboard or pointer**.
Every application is an independent **process**, and you can open **multiple
shell terminals** at once (these are userspace shells, not the kernel terminal).

## Features

| area | what it does |
|---|---|
| `wm` | tiling WM: workspaces, equal-column layout, `move_left/right`, `move_to_workspace`, `focus_*`, `window_at` |
| `shell` | userspace shell terminal: prompt, input line, builtins (`help`, `echo`, `clear`, `uname`, `exit`) |
| `process` | process registry: `spawn` / `kill` / `list` / `remove_by_window` |
| `input` | keyboard keys, mouse events, cursor |
| `render` | software renderer: rects, text (8×8 font), cursor |
| `Allde` | the desktop: spawn apps, route input to the focused shell, render frames |

## Commands (typed into a terminal)

* `new` — spawn another terminal,
* `list` — list running processes,
* `kill <pid>` — kill a process,
* `echo`, `clear`, `uname`, `help`, `exit` — shell builtins.

Keyboard: arrows move/focus windows, `Left`/`Right` reorder the tiling.

## Run

```sh
cd allde && cargo test          # unit tests (tiling, shell, processes, render)
cargo run                       # scripted session -> allde_frame.ppm
```

In the running OS the environment is launched from the in-system terminal with
`allde` (which switches input from the kernel terminal to the desktop).
