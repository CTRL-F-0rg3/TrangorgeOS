//! Wariant zwarty: ta sama szyna, ale mniejsza i czytelna.
//!
//! # Co zmieniam i dlaczego
//!
//! Wariant literalny ma wady, które wynikają z jego założeń, nie z gustu:
//!
//! * **Wysokość 821 px** na płótnie 856 px. Poniżej szyny zostaje 35 px
//!   pustki, a przerwa 159 px nad kółkiem powiadomień to puste miejsce.
//!   Cała szyna zajmuje 96% wysokości ekranu.
//! * **Ucięte słowa.** `apli` to `aplikacje` bez pięciu liter, a `powiad` to
//!   `powiadomienia` bez siedmiu. Żaden krój ani rozmiar tego nie naprawi -
//!   `powiadomienia` ma 129 px przy 17 px, a kolumna ma 64 px.
//! * **E1 to 166 px pigułki z tekstem, którego nie da się kliknąć.**
//!
//! Szerokości nie zmieniam z 78 px, bo to byłoby rozwiązanie pozorne:
//! `Aplikacje` w 14 px ma 62 px, więc szyna 68 px wymagałaby skrócenia
//! etykiety do `Aplik.`. Trzymam 76 px i pełne słowo, a wygrywa wysokość -
//! **821 px schodzi do 244 px**, czyli o 70%.
//!
//! # Zgodność
//!
//! Kolory, brak cieni i obramowań, promień kapsułki, zegar w lewym dolnym
//! rogu i pozycja panelu są identyczne jak w wariancie literalnym. Różni się
//! wyłącznie rozmiar szyny i treść, którą faktycznie da się przeczytać.

use crate::theme;
use uspace::gfx::text::{self, Align};

/// Lewa krawędź szyny - ta sama co w wariancie literalnym.
pub const RAIL_X: i32 = 15;
/// Szerokość szyny.
///
/// 76 px, a nie 78: `Aplikacje` w 14 px ma 62 px i musi zmieścić się z
/// sześcioma pikselami marginesu po każdej stronie. 68 px wymagałoby
/// skrócenia do `Aplik.`, a `Powiad.` i tak wchodzi dopiero w 64 px.
pub const RAIL_W: u32 = 76;

/// Wysokość jednego elementu szyny.
pub const ITEM_H: u32 = 52;
/// Odstęp między elementami.
pub const ITEM_GAP: i32 = 12;
/// Górna krawędź pierwszego elementu - jak w wariancie literalnym.
pub const FIRST_Y: i32 = 22;

/// Pozycja panelu: 14 px za prawą krawędzią szyny, jak w wariancie literalnym.
pub const PANEL_X: i32 = RAIL_X + RAIL_W as i32 + theme::PANEL_GAP;
/// Wysokość panelu, jak w wariancie literalnym.
pub const PANEL_Y: i32 = 302;
pub const PANEL_W: u32 = 338;
pub const PANEL_H: u32 = 508;
pub const PANEL_RADIUS: u32 = 27;

/// Promień kapsułki: połowa szerokości.
pub const PILL_RADIUS: u32 = RAIL_W / 2;

/// Pasek zegara - bez zmian względem wariantu literalnego.
pub const E5_X: i32 = 2;
pub const E5_Y: i32 = 805;
pub const E5_W: u32 = 108;
pub const E5_H: u32 = 38;
const E5_RADIUS: u32 = 9;


/// Element szyny: napis i jego pozycja.
pub struct Item {
    pub label: &'static str,
    pub y: i32,
}

/// Cztery elementy szyny, w kolejności od góry.
///
/// Pierwszy element odpowiada E1 ze wariantu literalnego, ale zamiast
/// pięciu uciętych linii niesie jedno słowo. Reszta zachowuje kolejność
/// i sens ze specyfikacji.
pub fn items() -> [Item; 4] {
    [
        Item { label: "Ekran", y: FIRST_Y },
        Item { label: "Aplikacje", y: FIRST_Y + (ITEM_H as i32 + ITEM_GAP) },
        Item { label: "Powiad.", y: FIRST_Y + 2 * (ITEM_H as i32 + ITEM_GAP) },
        Item { label: "Więcej", y: FIRST_Y + 3 * (ITEM_H as i32 + ITEM_GAP) },
    ]
}

/// Wysokość zajmowana przez szynę: cztery elementy i trzy przerwy.
pub fn rail_height() -> u32 {
    4 * ITEM_H + 3 * ITEM_GAP as u32
}

/// Akcje, które panel pokazuje zamiast zdania opisowego.
///
/// Zdanie z rzutu ("po kliknięciu więcej jest więcej opcji typu...") opisuje
/// te siedem rzeczy. Wypisanie ich jako listy zamienia ścianę tekstu w coś,
/// po co można kliknąć; opis zostaje w wariancie literalnym, w tym samym
/// miejscu, żeby porównanie obu wariantów było wartościowe.
pub const ACTIONS: [&str; 6] = [
    "Schowek",
    "Historia powiadomień",
    "Lupa",
    "Aplikacje w tle",
    "Ustawienia",
    "Monitor GPU",
];

/// Rysuje wariant zwarty w stanie zamkniętym.
pub fn draw_closed(buf: &mut [u32]) {
    draw_rail(buf);
}

/// Rysuje wariant zwarty w stanie otwartym.
///
/// W odróżnieniu od wariantu literalnego nic nie znika z szyny: cztery
/// elementy są informacyjne, a klikalne jest tylko "Więcej", które zostaje
/// na swoim miejscu. Zniknięcie kółka powiadomień ze wariantu literalnego
/// wynikało z tego, że było piątym elementem w pionowym układzie; tutaj
/// układ jest inny i ten sam zabieg nie miałby sensu.
pub fn draw_open(buf: &mut [u32]) {
    draw_panel(buf);
    draw_rail(buf);
}

/// Szyna: cztery kapsułki z etykietami wyśrodkowanymi w obu osiach.
fn draw_rail(buf: &mut [u32]) {
    for item in items() {
        theme::fill_rounded_rect(
            buf,
            RAIL_X,
            item.y,
            RAIL_W,
            ITEM_H,
            PILL_RADIUS,
            theme::SHAPE,
        );
        theme::draw_centered(
            buf,
            RAIL_X,
            item.y,
            RAIL_W,
            ITEM_H,
            theme::LABEL_SIZE,
            item.label,
            theme::TEXT,
        );
    }
    draw_clock(buf);
}

/// Pasek zegara - identyczny jak w wariancie literalnym.
fn draw_clock(buf: &mut [u32]) {
    theme::fill_rounded_rect(buf, E5_X, E5_Y, E5_W, E5_H, E5_RADIUS, theme::SHAPE);
    theme::draw_centered(
        buf,
        E5_X,
        E5_Y,
        E5_W,
        E5_H,
        theme::RAIL_SIZE,
        crate::dock::E5_TEXT,
        theme::TEXT,
    );
}

/// Panel: lista sześciu akcji zamiast ściany tekstu.
///
/// Wiersze są liczone z tekstu, a nie wpisane ręcznie, więc zmiana rozmiaru
/// pisma nie rozjedzie ich od panelu. Centrowanie w pionie jest takie samo jak
/// w wariancie literalnym: liczymy blok i dzielimy resztę na pół.
fn draw_panel(buf: &mut [u32]) {
    theme::fill_rounded_rect(
        buf,
        PANEL_X,
        PANEL_Y,
        PANEL_W,
        PANEL_H,
        PANEL_RADIUS,
        theme::SHAPE,
    );

    // Lewy margines tekstu: 16 px, żeby lista nie przylegała do zaokrąglonego
    // rogu panelu. To ten sam margines co w wariancie literalnym ma kolumna
    // tekstu 336 px na szerokości 338 px.
    const PAD: i32 = 16;
    let box_w = PANEL_W - (PAD as u32) * 2;
    let line_height = theme::COMPACT_LEADING;
    let block = ACTIONS.len() as u32 * line_height;
    let top = PANEL_Y + ((PANEL_H as i32 - block as i32) / 2);

    for (index, action) in ACTIONS.iter().enumerate() {
        text::draw_clipped(
            buf,
            theme::CANVAS_W,
            theme::CANVAS_H,
            PANEL_X + PAD,
            top + (index as i32 * line_height as i32),
            theme::LABEL_SIZE,
            action,
            theme::TEXT,
            box_w,
            Align::Left,
        );
    }
}
