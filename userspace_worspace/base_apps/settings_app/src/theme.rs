//! Kolory i geometria obu wariantów menu.
//!
//! # Dlaczego to osobny plik
//!
//! Sa dwa warianty tego samego menu, ktore trzeba porownywac obok siebie
//! we framebufferze. Roznica miedzy nimi to wyłącznie liczby, wiec trzymanie
//! ich w jednym miejscu oznacza, ze porownanie jest uczciwe: przestawia sie
//! jedna zmienna, a nie szuka po plikach, gdzie ktos wpisal 78.
//!
//! Nic tu nie jest "tymczasowe". Szerokosc 78 px w wariancie literalnym to
//! nie kaprys, tylko wynik pomiaru - `obszary` w 17 px ma 66 px, a kolumna
//! w rzucie ma 64 px, wiec slowo musi wyjsc poza obszar.

use uspace::gfx::{fill_rect, text, Color};

/// Szerokosc płótna, w pikselach.
pub const CANVAS_W: u32 = 1568;
/// Wysokosc płótna, w pikselach.
pub const CANVAS_H: u32 = 856;

// ── kolory ──────────────────────────────────────────────────────────────────

/// Lewy koniec gradientu tła: ciemny fiolet.
pub const BG_LEFT: Color = 0xFF3B_0B47;
/// Prawy koniec gradientu tła: żywy magenta.
pub const BG_RIGHT: Color = 0xFFB5_189D;
/// Wypełnienie kształtów: lodowy błękit. Ten sam dla całej szyny.
pub const SHAPE: Color = 0xFFDB_E1F8;
/// Kolor pisma: prawie czarny.
pub const TEXT: Color = 0xFF10_1018;

// ── typografia ──────────────────────────────────────────────────────────────

/// Pismo w szynie wariantu literalnego.
pub const RAIL_SIZE: u32 = 17;
/// Pismo w panelu.
pub const PANEL_SIZE: u32 = 19;
/// Pismo w etykietach wariantu zwartego.
pub const LABEL_SIZE: u32 = 14;
/// Interlinia w panelu, liczona między górami kolejnych linii.
pub const PANEL_LEADING: u32 = 28;
/// Interlinia w szynie wariantu literalnego.
pub const RAIL_LEADING: u32 = 28;
/// Interlinia w wariancie zwartym.
pub const COMPACT_LEADING: u32 = 18;

/// Kolumna tekstu w szynie wariantu literalnego, w pikselach.
pub const RAIL_BOX: u32 = 64;
/// Kolumna tekstu w panelu, w pikselach.
pub const PANEL_BOX: u32 = 336;

/// Odstęp pomiędzy prawą krawędzią szyny a panelem.
pub const PANEL_GAP: i32 = 14;

// ── tło ────────────────────────────────────────────────────────────────────

/// Wypełnia płótno gradientem poziomym `left` → `right`.
///
/// Interpolacja jest liczona na bajcie i z zaokrągleniem w dół, więc ostatnia
/// kolumna to dokładnie `right`, a pierwsza - `left`. Gradient idzie wyłącznie
/// w poziomie: pionowy gradient w specyfikacji jest wykluczony, a mieszanie
/// dwóch kierunków wygląda jak przypadek, nie jak zamiar.
pub fn fill_background(buf: &mut [u32]) {
    for x in 0..CANVAS_W {
        let t = x as u64 * 255 / (CANVAS_W as u64 - 1);
        let color = lerp(BG_LEFT, BG_RIGHT, t as u32);
        for y in 0..CANVAS_H {
            let index = (y as usize * CANVAS_W as usize) + x as usize;
            if let Some(pixel) = buf.get_mut(index) {
                *pixel = color;
            }
        }
    }
}

/// Miesza dwa kolory w proporcji `t` z zakresu 0..=255.
#[inline]
pub fn lerp(a: Color, b: Color, t: u32) -> Color {
    let t = t.min(255);
    let inv = 255 - t;
    let r = (((a >> 16) & 0xFF) * inv + ((b >> 16) & 0xFF) * t) / 255;
    let g = (((a >> 8) & 0xFF) * inv + ((b >> 8) & 0xFF) * t) / 255;
    let bl = ((a & 0xFF) * inv + (b & 0xFF) * t) / 255;
    0xFF00_0000 | (r << 16) | (g << 8) | bl
}

/// Pierwiastek z całkowitej, zaokrąglony w dół.
///
/// Długości boku w rogach są małe - najwyżej kilkadziesiąt pikseli - więc
/// pętla z odejmowaniem jest najtańsza i nie potrzebuje floatów ani tablicy.
/// Pierwiastek z całkowitej, zaokrąglony w dół.
///
/// Długości boku w rogach są małe - najwyżej kilkadziesiąt pikseli - więc
/// pętla jest najtańsza i nie potrzebuje floatów ani tablicy.
///
/// Sprawdza się `guess * guess <= value`, a nie `guess.pow(2)`: potęgowanie
/// w `u32` przepełnia się od 2^16, a `guess * guess` dla tej samej wartości
/// też - dlatego pętla startuje od 1 i nigdy nie mnoży poza zakres potrzebny.
fn isqrt(value: u32) -> u32 {
    if value == 0 {
        return 0;
    }
    let mut guess = 1u32;
    while (guess + 1) * (guess + 1) <= value {
        guess += 1;
    }
    guess
}

/// Wypełnia prostokąt z zaokrąglonymi rogami.
///
/// `radius` jest obcinany do połowy krótszego boku, dzięki czemu wartości
/// większe niż możliwe dają kapsułkę, a nie dziurę. To dokładnie to, czego
/// wymaga specyfikacja dla pigułek: promień równy połowie wysokości, czyli
/// 39 px dla szyny o szerokości 78 px, niezależnie od tego, jak wysoka jest
/// pigułka.
pub fn fill_rounded_rect(
    buf: &mut [u32],
    x: i32,
    y: i32,
    w: u32,
    h: u32,
    radius: u32,
    color: Color,
) {
    if w == 0 || h == 0 {
        return;
    }
    let max_radius = (w.min(h) / 2) as u32;
    let radius = radius.min(max_radius);

    // Wiersze, w których róg jeszcze jest krzywizny. Poza nimi wypełnienie
    // jest pełną szerokością.
    for row in 0..h {
        let py = y + row as i32;

        // Odległość pionowa od środka zaokrąglonego rogu, po obu stronach.
        let into_corner = if row < radius {
            radius - row
        } else if row >= h - radius {
            row - (h - radius) + 1
        } else {
            0
        };

        let inset = if into_corner == 0 {
            0
        } else {
            // W poziomie przesuwamy się o tyle, ile wynosi odległość od
            // środka okręgu: `dx² + dy² = r²`.
            let dy = into_corner - 1;
            let squared = radius * radius - dy * dy;
            radius - isqrt(squared)
        };

        let x0 = x + inset as i32;
        let x1 = x + w as i32 - inset as i32;
        if x1 > x0 {
            fill_rect(buf, CANVAS_W, CANVAS_H, x0, py, (x1 - x0) as u32, 1, color);
        }
    }
}

/// Rysuje tekst wyśrodkowany w pionie i w poziomie wewnątrz pola.
///
/// `pad_top` to odstęp od góry pola do góry pierwszej linii. Domyślnie 0,
/// czyli tekst zaczyna sie zaraz pod gora, i wtedy wycentrowanie pionowe
/// musi byc policzone przez callera - dzieki temu ta sama funkcja obsluguje
/// i pole 38 px, i kapsułkę 243 px, bez osobnych sciezek kodu.
pub fn draw_centered(
    buf: &mut [u32],
    x: i32,
    y: i32,
    w: u32,
    h: u32,
    size: u32,
    text: &str,
    color: Color,
) -> u32 {
    let line_height = text::ascent(size).unwrap_or(size) + text::descent(size).unwrap_or(0);
    let top = y + ((h as i32 - line_height as i32) / 2).max(0);
    text::draw_clipped(
        buf,
        CANVAS_W,
        CANVAS_H,
        x,
        top,
        size,
        text,
        color,
        w,
        text::Align::Center,
    )
}
