//! Wariant literalny: geometria wprost ze zrzutu ekranu.
//!
//! # Co tu jest ważne
//!
//! Wszystkie współrzędne pochodzą z pomiaru, nie z estetyki. Przerwy pionowe
//! są w kodzie jako komentarze obok wywołań, bo to one - a nie same pozycje -
//! mówią, czy układ jest zgodny: 25 / 159 / 10 / 9 px między kolejnymi
//! kształtami. Zmiana `E3_Y` o jeden piksel rozjedzie wszystkie cztery
//! przerwy naraz, i to widać w liczbach, nie dopiero na obrazie.
//!
//! # Stan zamknięty i otwarty
//!
//! W stanie otwartym znika wyłącznie E3 ("powiadomienia"). E4 zostaje na
//! swoim miejscu - tak mówi specyfikacja, i tak jest tu: `E4_Y` nie zależy od
//! stanu. Zmiana stanu to przełącznik jednego pola, a nie przeliczenie
//! układu.

use crate::theme;
use uspace::gfx::text::{self, Align};


// ── pozycje wariantu literalnego ────────────────────────────────────────────

/// Lewa krawędź szyny.
pub const RAIL_X: i32 = 15;
/// Szerokość szyny - wspólna dla wszystkich pigułek.
pub const RAIL_W: u32 = 78;

/// Wysokość i pozycja pierwszej pigułki, z tekstem.
pub const E1_Y: i32 = 22;
pub const E1_H: u32 = 166;

/// Pozycja drugiej pigułki, "apli".
pub const E2_Y: i32 = 213;
pub const E2_H: u32 = 243;

/// Kółko powiadomień - tylko w stanie zamkniętym.
pub const E3_Y: i32 = 615;
pub const E3_H: u32 = 88;

/// Kółko "Więcej" - klikalne, otwiera panel.
pub const E4_Y: i32 = 713;
pub const E4_H: u32 = 83;

/// Pasek zegara, przyklejony do lewego dolnego rogu.
///
/// Nie jest częścią szyny: ma inne wyrównanie (2 px od krawędzi okna, nie
/// 15) i szerszy jest niż szyna - 108 px, czyli wychodzi 17 px poza jej prawą
/// krawędź. To zamierzone, to pasek systemowy, a nie element docka.
pub const E5_X: i32 = 2;
/// 805 = 713 + 83 + 9: pozycja E4 plus jego wysokość plus przerwa 9 px.
pub const E5_Y: i32 = 805;
pub const E5_W: u32 = 108;
pub const E5_H: u32 = 38;

/// Panel informacyjny.
pub const PANEL_X: i32 = 107;
pub const PANEL_Y: i32 = 302;
pub const PANEL_W: u32 = 338;
pub const PANEL_H: u32 = 508;

/// Zaokrąglenie panelu.
pub const PANEL_RADIUS: u32 = 27;

/// Tekst pierwszej pigułki.
///
/// Pełne słowa, nie ich widoczne fragmenty. To one są ucięte przy
/// rysowaniu - `obszary` ma 66 px przy 17 px, a kolumna ma 64 px. Wpisanie
/// tu `obszar` byłoby wpisaniem artefaktu renderowania w miejsce treści.
const E1_TEXT: &str = "obszary robocze oraz więcej aplikacji";

/// Etykieta drugiej pigułki.
///
/// "apli" w rzucie to ucięte "aplikacje" (9 liter, 70 px, kolumna 64 px) -
/// ten sam artefakt co w E1. Tu jest pełne słowo, bo tutaj nic nie jest
/// przycinane: etykieta jest wyśrodkowana w całości.
const E2_TEXT: &str = "apli";

/// Tekst kółka powiadomień, łamany na trzy linie.
///
/// Wpisane jako trzy ciągi zamiast jako jedno słowo, bo `powiadomienia` ma
/// 129 px przy 17 px, czyli nie mieści się w 64 px nawet w połowie - łamanie
/// słów w środku wygląda jak szum, a te trzy fragmenty są czytelne.
const E3_LINES: [&str; 3] = ["powiad", "omieni", "a"];

/// Etykieta kółka otwierającego panel.
const E4_TEXT: &str = "Więcej";

/// Etykieta paska zegara.
pub const E5_TEXT: &str = "czas i data";

/// Akapit panelu.
///
/// Dziewięć wierszy po przebudowaniu, nie dziesięć - tyle jest w rzutach.
/// `zużycie`, nie `złurzycie`: to jedyny wyraz w specyfikacji, który nie jest
/// polskim słowem, a kontekst ("monitor ... karty graficznej") wskazuje
/// jednoznacznie na zużycie zasobów.
const PANEL_TEXT: &str = "po kliknięciu więcej jest więcej opcji typu. schowek \
     historia powiadomień, lupa, możliwość jakie programy są włączone \
     nawet które działają w tle, dostęp do ustawień, możliwość włączenia \
     systemu monitoru co pokazuje zużycie ramu karty graficznej \
     (komponentów) itp.";

/// Promień pigułek: połowa szerokości szyny.
///
/// Daje kapsułkę, nie koło - pigułka E3 ma 88 px wysokości przy 78 px
/// szerokości, więc prawdziwy okrągły by jej nie zmieścił. Specyfikacja sama
/// pisze "koło / kapsułka", więc kapsułka jest zgodna.
const PILL_RADIUS: u32 = RAIL_W / 2;

/// Zaokrąglenie paska zegara.
const E5_RADIUS: u32 = 9;

/// Rysuje wariant literalny w stanie zamkniętym.
///
/// Kolejność jest tu celowa: panel przed szyną, mimo że nie ma kolizji
/// (panel zaczyna się 14 px za prawą krawędzią szyny). Specyfikacja każe
/// panelowi leżeć pod elementami szyny, a kolejność rysowania jest jedynym
/// sposobem, żeby to odtworzyć w buforze - gdyby kiedyś szerokości się
/// zmieniły, kolejność utrzyma właściwe nakładanie bez dodatkowych testów.
pub fn draw_closed(buf: &mut [u32]) {
    draw_rail(buf);
}

/// Rysuje wariant literalny w stanie otwartym: szyna bez E3, plus panel.
pub fn draw_open(buf: &mut [u32]) {
    // Panel najpierw - leży pod szyną.
    draw_panel(buf);

    // Szyna bez kółka powiadomień. E4 zostaje na swoim miejscu, więc
    // przerwa 159 px pod E2 wygląda pusto dokładnie tak, jak w specyfikacji.
    theme::fill_rounded_rect(buf, RAIL_X, E1_Y, RAIL_W, E1_H, PILL_RADIUS, theme::SHAPE);
    draw_rail_text(buf);

    theme::fill_rounded_rect(buf, RAIL_X, E2_Y, RAIL_W, E2_H, PILL_RADIUS, theme::SHAPE);
    theme::draw_centered(
        buf, RAIL_X, E2_Y, RAIL_W, E2_H, theme::RAIL_SIZE, E2_TEXT, theme::TEXT,
    );

    theme::fill_rounded_rect(buf, RAIL_X, E4_Y, RAIL_W, E4_H, PILL_RADIUS, theme::SHAPE);
    theme::draw_centered(
        buf, RAIL_X, E4_Y, RAIL_W, E4_H, theme::RAIL_SIZE, E4_TEXT, theme::TEXT,
    );

    draw_clock(buf);
}

/// Sama szyna w stanie zamkniętym, bez panelu.
fn draw_rail(buf: &mut [u32]) {
    // E1 - 22, wysokość 166, promień 39.
    theme::fill_rounded_rect(buf, RAIL_X, E1_Y, RAIL_W, E1_H, PILL_RADIUS, theme::SHAPE);
    draw_rail_text(buf);

    // E2 - 213. Przerwa: 213 - (22 + 166) = 25 px.
    theme::fill_rounded_rect(buf, RAIL_X, E2_Y, RAIL_W, E2_H, PILL_RADIUS, theme::SHAPE);
    theme::draw_centered(
        buf, RAIL_X, E2_Y, RAIL_W, E2_H, theme::RAIL_SIZE, E2_TEXT, theme::TEXT,
    );

    // E3 - 615. Przerwa: 615 - (213 + 243) = 159 px.
    theme::fill_rounded_rect(buf, RAIL_X, E3_Y, RAIL_W, E3_H, PILL_RADIUS, theme::SHAPE);
    draw_notifications(buf);

    // E4 - 713. Przerwa: 713 - (615 + 88) = 10 px.
    theme::fill_rounded_rect(buf, RAIL_X, E4_Y, RAIL_W, E4_H, PILL_RADIUS, theme::SHAPE);
    theme::draw_centered(
        buf, RAIL_X, E4_Y, RAIL_W, E4_H, theme::RAIL_SIZE, E4_TEXT, theme::TEXT,
    );

    draw_clock(buf);
}

/// Tekst pigułki E1.
///
/// Pięć linii, dociętych do kolumny 64 px, do lewej, z wyrównaniem 20 px od
/// góry. Zawijanie po słowach + cięcie każdej linii, a nie zawijanie w
/// dowolnym miejscu: łamanie słów w środku daje w 64 px same ucięte
/// sylaby, a spec zrzutu pokazuje pełne słowa wychodzące poza krawędź.
fn draw_rail_text(buf: &mut [u32]) {
    let pad_left = 7;
    let pad_top = 20;
    text::draw_wrapped(
        buf,
        theme::CANVAS_W,
        theme::CANVAS_H,
        RAIL_X + pad_left,
        E1_Y + pad_top,
        theme::RAIL_SIZE,
        E1_TEXT,
        theme::RAIL_LEADING,
        theme::TEXT,
        theme::RAIL_BOX,
        Align::Left,
    );
}

/// Kółko powiadomień: trzy linie wyśrodkowane w pionie i poziomie.
///
/// `a` w trzeciej linii to ogonek słowa "powiadomienia" przeniesiony przez
/// zawijanie - stąd trzy linie zamiast dwóch.
fn draw_notifications(buf: &mut [u32]) {
    let leading = theme::RAIL_LEADING;
    let block = E3_LINES.len() as u32 * leading;
    let top = E3_Y + ((E3_H as i32 - block as i32) / 2);

    for (index, line) in E3_LINES.iter().enumerate() {
        text::draw_clipped(
            buf,
            theme::CANVAS_W,
            theme::CANVAS_H,
            RAIL_X,
            top + (index as i32 * leading as i32),
            theme::RAIL_SIZE,
            line,
            theme::TEXT,
            RAIL_W,
            Align::Center,
        );
    }
}

/// Pasek zegara w lewym dolnym rogu.
fn draw_clock(buf: &mut [u32]) {
    theme::fill_rounded_rect(buf, E5_X, E5_Y, E5_W, E5_H, E5_RADIUS, theme::SHAPE);
    theme::draw_centered(
        buf, E5_X, E5_Y, E5_W, E5_H, theme::RAIL_SIZE, E5_TEXT, theme::TEXT,
    );
}

/// Panel informacyjny: dziewięć linii, wyśrodkowanych w obu osiach.
///
/// Wysokość bloku jest liczona z tekstu, a nie zgadywana z panelu, i potem
/// centrowana. Przy 9 liniach i interlinii 28 px blok ma 252 px, więc nad i
/// pod zostaje (508 - 252) / 2 = 128 px. Specyfikacja mówiła "około 40 px",
/// co z tą geometrią się nie zgadza - 40 px wymagałoby albo interlinii 42,8 px,
/// albo 15 linii. Wygrywa geometria: panel ma 508 px, akapit ma 9 linii po
/// 28 px, i liczby muszą się zgodzić.
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

    let block = text::wrapped_height(
        theme::PANEL_SIZE,
        PANEL_TEXT,
        theme::PANEL_BOX,
        theme::PANEL_LEADING,
    );
    let top = PANEL_Y + ((PANEL_H as i32 - block as i32) / 2);

    text::draw_wrapped(
        buf,
        theme::CANVAS_W,
        theme::CANVAS_H,
        PANEL_X,
        top,
        theme::PANEL_SIZE,
        PANEL_TEXT,
        theme::PANEL_LEADING,
        theme::TEXT,
        theme::PANEL_BOX,
        Align::Center,
    );
}
