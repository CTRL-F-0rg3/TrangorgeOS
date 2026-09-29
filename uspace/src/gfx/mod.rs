//! Graphical environment built on the kernel-provided framebuffer.
//!
//! The chain mirrors the driver-space compositor protocol (`gfx-protocol` /
//! `gfx-server`) and the Vise LG runtime: surfaces are composited in z-order
//! into the kernel framebuffer, and simple software rendering (rects + text)
//! stands in for the GPU pipeline on this reference implementation.

pub mod compositor;
pub mod render;
pub mod app;

// Wygenerowane w `tools/genfont.py`. `font_atlas` to same metryki wrodzone do
// kodu, a `text` rysuje je. Oba sa w repo, wiec zbudowanie `uspace` nie
// wymaga Pythona ani pliku TTF.
pub mod font_atlas;
pub mod text;

pub use compositor::{Compositor, Surface};
pub use render::{draw_rect, fill_rect, rgb, Color};
pub use app::draw_demo_window;
pub use text::{draw_clipped, draw_wrapped, measure, Align};
