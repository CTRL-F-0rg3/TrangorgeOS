//! Warstwa tekstowa: UTF-8, pomiar, przycinanie i zawijanie.
//!
//! # Po co to istnieje
//!
//! `render::draw_text` rysuje fontem 8x8 o zakresie ASCII 32..126. Ma dwie
//! wady, z ktorych kazda blokuje to menu: bajty spoza zakresu sa **po cichu**
//! pomijane, wiec `powiadomienia` wypisuje sie jako `powiadomienia` bez `ó`,
//! a rozmiaru 17-19 px nie da sie w ogole uzyskac.
//!
//! Ta warstwa rysuje z atlasu wygenerowanego przez `tools/genfont.py`, wiec w
//! runtime nie ma rasteryzera, floatow ani alokacji.
//!
//! # Dwa tryby, bo dwa różne cele
//!
//! * [`draw_clipped`] nie zawija. Przerywa na krawedzi pola i obcina piksele.
//!   To jest tryb wariantu "literalnego", w ktorym `obszary` w kolumnie 64 px
//!   ma wyswietlic `obszar` - slowo ma wyjsc poza obszar, nie przeskoczyc na
//!   kolejny wiersz.
//! * [`draw_wrapped`] zawija po slowach i zwraca wysokosc zuzytego bloku,
//!   dzieki czemu blok mozna wycentrowac w pionie bez zgadywania rozmiaru.
//!
//! # Pozycjonowanie
//!
//! `y` oznacza **gore linii**, nie linie bazowa. Linia bazowa lezy o
//! `ascent(size)` nizej. Podanie w `y` bezposrednio linii bazowej jest
//! pospolitym bledem, bo tekst z `ó` wychodzi wtedy o wysokosc malej litery
//! wyzej niz powinien.

use super::font_atlas::{self, Glyph, BITMAPS};

pub use super::render::Color;

/// Wyrównanie poziome bloku tekstu.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Align {
    /// Do lewej krawedzi pola.
    Left,
    /// Wycentrowane w polu.
    Center,
}

/// Wysokosc wstepujaca kroju w danym rozmiarze.
///
/// `None` dla rozmiaru spoza atlasu - wtedy nie da sie policzyc pozycji
/// bazowej, a zgadywanie `size` daloby tekst przesuniety o kilka pikseli.
pub const fn ascent(size: u32) -> Option<u32> {
    match size {
        14 => Some(font_atlas::ASCENT_14),
        17 => Some(font_atlas::ASCENT_17),
        19 => Some(font_atlas::ASCENT_19),
        _ => None,
    }
}

/// Glebokosc zejscia kroju w danym rozmiarze, dodatnia.
pub const fn descent(size: u32) -> Option<u32> {
    match size {
        14 => Some(font_atlas::DESCENT_14),
        17 => Some(font_atlas::DESCENT_17),
        19 => Some(font_atlas::DESCENT_19),
        _ => None,
    }
}

/// Nastepny znak UTF-8 wraz z reszta napisu.
///
/// Zwraca `None` na koncu. Uszkodzona sekwencja zwraca `U+FFFD` i przesuwa
/// sie o jeden bajt, dzieki czemu renderer nie wpada w petle nieskonczona -
/// to jedyny przypadek, w ktorym `char_indices` tez zawodzi.
fn next_char(s: &str) -> Option<(char, &str)> {
    let bytes = s.as_bytes();
    if bytes.is_empty() {
        return None;
    }

    let first = bytes[0];
    let width = match first {
        0x00..=0x7F => 1,
        0xC0..=0xDF => 2,
        0xE0..=0xEF => 3,
        0xF0..=0xF7 => 4,
        // 0x80..=0xBF to kontynuacja, 0xF8+ nie jest UTF-8 w ogole.
        _ => return Some(('\u{FFFD}', &s[1..])),
    };

    if width == 1 {
        return Some((first as char, &s[1..]));
    }
    if bytes.len() < width {
        return Some(('\u{FFFD}', &s[1..]));
    }
    match s.get(..width).and_then(|part| part.chars().next()) {
        Some(ch) => Some((ch, &s[width..])),
        None => Some(('\u{FFFD}', &s[1..])),
    }
}

/// Punkt kodowy glifu zastepczego.
///
/// Musi być zsynchronizowany z `FALLBACK` w `tools/genfont.py` - generator
/// rysuje tę pozycję jako wypełniony prostokąt. Jeśli generator ją zmieni, a
/// tutaj zostanie stare, `glyph()` przestanie znajdować zamiennik i cicho nic nie
/// narysuje, czyli wrócimy do braku, który ta funkcja ma naprawić.
const FALLBACK_CODEPOINT: u32 = 0x25A0; // '■'

/// Glif dla punktu kodowego, albo `None` gdy rozmiaru nie ma w atlasie.
///
/// Znak spoza zakresu atlasu dostaje glif zastepczy - wypełniony prostokąt.
/// Pomijanie go po cichu byłoby najgorszą z trzech możliwości: rysownik
/// widziałby napis bez jednej litery i nie miałby skąd wiedzieć, co się stało.
/// Prostokąt mówi wprost, że tej litery tu nie ma.
///
/// `None` znaczy tylko "nie ma takiego rozmiaru w atlasie", czyli błąd
/// programisty, a nie sytuację do obsługi.
fn glyph(size: u32, codepoint: u32) -> Option<&'static Glyph> {
    let table = font_atlas::table_for(size)?;
    // Tabela jest posortowana po punkcie kodowym, wiec wyszukiwanie
    // dwudzielne jest tu poprawne a nie tylko szybkie.
    let index = match table.binary_search_by_key(&codepoint, |g| g.codepoint) {
        Ok(found) => found,
        Err(_) => table.binary_search_by_key(&FALLBACK_CODEPOINT, |g| g.codepoint).ok()?,
    };
    table.get(index)
}

/// Szerokosc napisu w pikselach - suma posuwu, nie suma szerokosci masek.
///
/// To rozroznienie ma znaczenie: `i` jest w wielu kroju wstepujacej, a `f`
/// dolu, wiec wiersz mialby za szerokosc o kilka pikseli, gdyby liczyc
/// obwiednie. Zwracana szerokosc to odleglosc, o ktora przesuwa sie pismo.
pub fn measure(size: u32, text: &str) -> u32 {
    if font_atlas::table_for(size).is_none() {
        return 0;
    }

    let mut total = 0u32;
    let mut rest = text;
    while let Some((ch, tail)) = next_char(rest) {
        if let Some(g) = glyph(size, ch as u32) {
            total += g.advance as u32;
        }
        rest = tail;
    }
    total
}

/// Ile znakow z `text` zmiesci sie w `box_w` pikselach.
///
/// Uzywana przez testy do sprawdzenia, gdzie lezy granica ucięcia, i przez
/// nic w kodzie produkcyjnym - dlatego kompiluje sie tylko z testami. Trzymana
/// osobno, bo odpowiedź na "co się zmieści" jest różna od odpowiedzi na
/// "co zostanie narysowane": tu liczymy znaki, tam liczymy piksele.
#[cfg(test)]
fn prefix_fitting(size: u32, text: &str, box_w: u32) -> (usize, u32) {
    let mut used = 0u32;
    let mut count = 0usize;
    let mut rest = text;
    while let Some((ch, tail)) = next_char(rest) {
        let advance = match glyph(size, ch as u32) {
            Some(g) => g.advance as u32,
            None => 0,
        };
        if used + advance > box_w {
            break;
        }
        used += advance;
        count += ch.len_utf8();
        rest = tail;
    }
    (count, used)
}

/// Wymiesza jeden piksel: `src` na tle `dst`, przy kryciu `alpha` 0..=255.
///
/// Najwyzszy bajt `dst` (alfa XRGB) jest zachowywany. Framebuffer ma ten bajt
/// ustawiony na `0xFF` i nikt go nie czyta jako znaczenia, ale nadpisanie go
/// mieszaniem dalo by piksel, ktory w nastepnym kroku wyglada inaczej niz
/// ten sam kolor - czyli niespodziewana niespojnosc w miejscu, gdzie nie ma
/// zadnego powodu jej szukac.
#[inline]
fn blend(dst: &mut u32, src: Color, alpha: u32) {
    if alpha == 0 {
        return;
    }
    let inv = 255 - alpha;
    let d = *dst;
    let r = (((d >> 16) & 0xFF) * inv + ((src >> 16) & 0xFF) * alpha) / 255;
    let g = (((d >> 8) & 0xFF) * inv + ((src >> 8) & 0xFF) * alpha) / 255;
    let b = ((d & 0xFF) * inv + (src & 0xFF) * alpha) / 255;
    *dst = (d & 0xFF00_0000) | (r << 16) | (g << 8) | b;
}

/// Jedna maska 4 bpp: dwa piksele w bajcie, gora najpierw.
#[inline]
fn mask_alpha(bitmaps: &[u8], offset: usize, index: usize) -> u32 {
    let byte = bitmaps[offset + (index >> 1)] as u32;
    let nibble = if index & 1 == 0 { byte >> 4 } else { byte & 0x0F };
    // 15 * 17 = 255, wiec 16 poziomow rozciaga sie na pelny zakres 0..=255.
    nibble * 17
}

/// Narysuje maske jednego glifu, obcinajac ja do prostokata docelowego.
///
/// Obciecie jest na poziomie pikseli, nie znakow: glif lezacy na krawedzi
/// pola wyswietla sie w czesci. Dopiero obciecie co do piksela daje efekt
/// "slowo wychodzi poza obszar" zamiast "slowo znika".
#[allow(clippy::too_many_arguments)]
fn blit_glyph(
    buf: &mut [u32],
    w: u32,
    h: u32,
    g: &Glyph,
    pen_x: i32,
    baseline_y: i32,
    color: Color,
    clip_x0: i32,
    clip_y0: i32,
    clip_x1: i32,
    clip_y1: i32,
) {
    if g.width == 0 || g.height == 0 || g.length == 0 {
        return;
    }

    let gx = pen_x + g.bearing_x as i32;
    let gy = baseline_y - g.bearing_y as i32;
    let gw = g.width as i32;
    let gh = g.height as i32;

    // Prostokat maski po obcięciu: przecięcie z maską i z polem, plus z buforem.
    let x0 = gx.max(clip_x0).max(0);
    let y0 = gy.max(clip_y0).max(0);
    let x1 = (gx + gw).min(clip_x1).min(w as i32);
    let y1 = (gy + gh).min(clip_y1).min(h as i32);
    if x1 <= x0 || y1 <= y0 {
        return;
    }

    for py in y0..y1 {
        let row = (py - gy) as usize * g.width as usize;
        for px in x0..x1 {
            let index = row + (px - gx) as usize;
            let alpha = mask_alpha(BITMAPS, g.offset as usize, index);
            let slot = (py as u32 * w + px as u32) as usize;
            if let Some(pixel) = buf.get_mut(slot) {
                blend(pixel, color, alpha);
            }
        }
    }
}

/// Rysuje jeden wiersz, przycinajac go do pola o szerokosci `box_w`.
///
/// Zwraca szerokosc narysowanego tekstu. Znaki, ktore sie nie zmiescily, sa
/// obcinane piksel po pikselu - nie sa przeskakiwane.
pub fn draw_clipped(
    buf: &mut [u32],
    w: u32,
    h: u32,
    x: i32,
    y: i32,
    size: u32,
    text: &str,
    color: Color,
    box_w: u32,
    align: Align,
) -> u32 {
    let Some(asc) = ascent(size) else {
        return 0;
    };

    let full = measure(size, text);
    let start_x = match align {
        Align::Left => x,
        Align::Center => x + ((box_w as i32 - full as i32) / 2),
    };
    let baseline_y = y + asc as i32;

    // Pole obcięcia: szerokość `box_w` od lewej krawędzi wiersza, nie od
    // wyrównanego początku tekstu. Dla `Center` liczone od `x`, bo inaczej
    // środek przesuwałby się z każdym wycentrowanym wierszem.
    let clip_x0 = x;
    let clip_x1 = x + box_w as i32;
    let clip_y0 = y;
    let clip_y1 = y + (asc + descent(size).unwrap_or(asc)) as i32;

    let mut pen = start_x;
    let mut rest = text;
    while let Some((ch, tail)) = next_char(rest) {
        let Some(g) = glyph(size, ch as u32) else {
            rest = tail;
            continue;
        };
        blit_glyph(buf, w, h, g, pen, baseline_y, color, clip_x0, clip_y0, clip_x1, clip_y1);
        pen += g.advance as i32;
        rest = tail;
    }

    full
}

/// Dzieli napis na wierszy po slowach, bez ligatur i bez dzielenia wyrazow.
///
/// Odpowiada temu, co robi przegladarka dla `overflow-wrap: normal`, i jest
/// tym samym zachowaniem, ktore zmierzylem przy identyfikacji kroju: 19 px w
/// kolumnie 336 px daje dziewiec linii.
///
/// Slowa sa najpierw zbierane jako zakresy bajtowe, a linie sklejane z nich
/// przez `line_start..line_end`. Sklejanie napisow byloby wygodniejsze, ale
/// wymaga alokacji - a wiersz musi byc wypelniony w calosci, zanim wiadomo,
/// czy kolejne slowo sie w nim zmiesci.
fn wrap_lines(size: u32, text: &str, box_w: u32) -> Vec<&str> {
    // Zbierz slowa jako pary bajtowych zakresow. Spacje miedzy nimi sa
    // pomijane; `gap` mowi, ile szerokosci zajmie jedna spacja przy sklejaniu.
    let mut words: Vec<(usize, usize)> = Vec::new();
    let mut word_start = 0usize;
    for (index, byte) in text.bytes().enumerate() {
        if byte == b' ' {
            if index > word_start {
                words.push((word_start, index));
            }
            word_start = index + 1;
        }
    }
    if word_start < text.len() {
        words.push((word_start, text.len()));
    }

    // Napis ze samymi spacjami albo pusty: jedna pusta linia, zeby caller
    // dostal cos do narysowania i nie musial obslugiwać pustego wektora.
    if words.is_empty() {
        return vec![text];
    }

    let gap = measure(size, " ");

    let mut lines = Vec::new();
    let mut line_start = words[0].0;
    let mut line_end = words[0].1;
    let mut line_w = measure(size, &text[words[0].0..words[0].1]);

    for &(start, end) in &words[1..] {
        let word_w = measure(size, &text[start..end]);
        if line_w + gap + word_w <= box_w {
            // Mieści się: linia rośnie o spację i słowo, czyli zakres
            // kończy się tam, gdzie kończy się słowo.
            line_end = end;
            line_w += gap + word_w;
        } else {
            // Nie mieści się: domykamy bieżącą i zaczynamy nową od tego
            // słowa. Spacja, ktora je poprzedzila, zostaje poza linia.
            lines.push(&text[line_start..line_end]);
            line_start = start;
            line_end = end;
            line_w = word_w;
        }
    }

    lines.push(&text[line_start..line_end]);
    lines
}

/// Rysuje akapit z zawijaniem po slowach.
///
/// Zwraca wysokosc zuzytego bloku w pikselach, dzieki czemu caller moze
/// wycentrowac akapit w pionie bez zgadywania. `line_height` jest odstepem
/// miedzy górami kolejnych linii i moze byc wiekszy od naturalnej wysokosci
/// kroju - tak uzyskuje sie rozluźniony interlinia typograficzny.
pub fn draw_wrapped(
    buf: &mut [u32],
    w: u32,
    h: u32,
    x: i32,
    y: i32,
    size: u32,
    text: &str,
    line_height: u32,
    color: Color,
    box_w: u32,
    align: Align,
) -> u32 {
    if ascent(size).is_none() {
        return 0;
    }

    let lines = wrap_lines(size, text, box_w);
    for (index, line) in lines.iter().enumerate() {
        let row_y = y + (index as u32 * line_height) as i32;
        draw_clipped(buf, w, h, x, row_y, size, line, color, box_w, align);
    }
    lines.len() as u32 * line_height
}

/// Wysokosc akapitu bez rysowania - do wycentrowania w pionie.
pub fn wrapped_height(size: u32, text: &str, box_w: u32, line_height: u32) -> u32 {
    if ascent(size).is_none() {
        return 0;
    }
    wrap_lines(size, text, box_w).len() as u32 * line_height
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Pelny tekst panelu, tak jak go podaje zrzut ekranu.
    ///
    /// Test ponizej trzyma go w jednym miejscu, bo dlatego w ogole wiemy,
    /// czym jest rozmiar kroja: to zdanie zawija sie na dokladnie 9 linii
    /// tylko przy 19 px i 336 px. Zmiana rozmiaru albo kolumny rozbija ten
    /// uklad, wiec test jest jednoczesnie zabezpieczeniem rozmiaru fontu.
    const PANEL: &str = "po kliknięciu więcej jest więcej opcji typu. schowek \
         historia powiadomień, lupa, możliwość jakie programy są włączone \
         nawet które działają w tle, dostęp do ustawień, możliwość włączenia \
         systemu monitoru co pokazuje zużycie ramu karty graficznej \
         (komponentów) itp.";

    /// Kolumna tekstu w panelu, w pikselach.
    const PANEL_BOX: u32 = 336;
    /// Rozmiar pisma w panelu, w pikselach.
    const PANEL_SIZE: u32 = 19;
    /// Odstęp między górami linii w panelu, w pikselach.
    const PANEL_LEADING: u32 = 28;

    /// Kolumna tekstu w szynie wariantu literalnego.
    const RAIL_BOX: u32 = 64;
    /// Rozmiar pisma w szynie.
    const RAIL_SIZE: u32 = 17;

    #[test]
    fn panel_wraps_to_exactly_nine_lines() {
        // To jest test identyfikacji kroju, nie test czytelności. Dziewięć
        // linii wychodzi dla 19 px w kolumnie 336 px; dla 17 px w tej samej
        // kolumnie wychodzi ich osiem.
        let lines = wrap_lines(PANEL_SIZE, PANEL, PANEL_BOX);
        assert_eq!(
            lines.len(),
            9,
            "9 linii to podpis kroju: zmiana rozmiaru lub kolumny psuje układ"
        );
    }

    #[test]
    fn panel_breaks_where_the_screenshot_breaks() {
        // Dwa pierwsze wiersze z rzutu sa najdluzsze, wiec to one rozstrzygaja
        // szerokosc kolumny. Gdyby kolumna byla szersza, trzeci wiersz
        // pochlonil by slowo "jakie".
        let lines = wrap_lines(PANEL_SIZE, PANEL, PANEL_BOX);
        assert_eq!(lines[0], "po kliknięciu więcej jest więcej");
        assert_eq!(lines[1], "opcji typu. schowek historia");
        assert_eq!(lines[2], "powiadomień, lupa, możliwość");
    }

    #[test]
    fn panel_height_is_nine_leadings() {
        assert_eq!(
            wrapped_height(PANEL_SIZE, PANEL, PANEL_BOX, PANEL_LEADING),
            9 * PANEL_LEADING
        );
    }

    #[test]
    fn rail_words_overflow_the_column_as_designed() {
        // Wariant literalny ma pokazac `obszary` jako `obszar`: slowo
        // wychodzi poza kolumne, a nie przeskakuje do kolejnego wiersza.
        let full = measure(RAIL_SIZE, "obszary");
        assert!(full > RAIL_BOX, "`obszary` musi nie mieścić się w {RAIL_BOX} px");

        let (chars, width) = prefix_fitting(RAIL_SIZE, "obszary", RAIL_BOX);
        assert!(width <= RAIL_BOX, "prefiks musi mieścić się w kolumnie");
        assert!(chars < "obszary".len(), "prefiks musi być ucięty");
        assert_eq!(&"obszary"[..chars], "obszar");
    }

    #[test]
    fn a_word_that_fits_is_not_clipped() {
        // `więcej` mieści się w całości, wiec nie wolno go uciąć: to ta sama
        // linia co w rzutie, i służy jako kontrast dla testu wyżej.
        let (chars, _) = prefix_fitting(RAIL_SIZE, "więcej", RAIL_BOX);
        assert_eq!(&"więcej"[..chars], "więcej");
    }

    #[test]
    fn polish_letters_have_advances() {
        // `render::draw_text` pomija te bajty po cichu, wiec `powiadomienia`
        // wypisuje się tam jako `powiadomienia` bez `ó`. Tu każda litera ma
        // dodatni posuw, a caly wyraz ma tyle, ile powinien.
        for ch in "ąćęłńóśźżĄĆĘŁŃÓŚŹŻ".chars() {
            assert!(
                measure(RAIL_SIZE, &ch.to_string()) > 0,
                "znak {ch:?} nie ma posuwu - jest poza zakresem?"
            );
        }
        assert_eq!(measure(RAIL_SIZE, "powiadomienia"), 129);
    }

    #[test]
    fn the_clipping_pill_shows_a_prefix_of_powiadomienia() {
        // Pierwszy wiersz kółka z rzutu to `powiad`; reszta wychodzi poza
        // kolumne. Dla 13 liter i 64 px to jedyne miejsce, gdzie kończy się
        // napis.
        let (chars, _) = prefix_fitting(RAIL_SIZE, "powiadomienia", RAIL_BOX);
        assert_eq!(&"powiadomienia"[..chars], "powiad");
    }

    #[test]
    fn every_rasterised_size_has_vertical_metrics() {
        for size in font_atlas::ALL_SIZES {
            assert!(ascent(size).is_some(), "brak wstepujacej dla {size} px");
            assert!(descent(size).is_some(), "brak zejscia dla {size} px");
            assert!(ascent(size).unwrap() > 0);
        }
    }

    #[test]
    fn an_unknown_size_measures_zero_instead_of_guessing() {
        // Rozmiar spoza atlasu to blad programisty, nie sytuacja do obslugi.
        // Zgadywanie `size` dałoby tekst przesuniety o kilka pikseli w bazowej.
        assert_eq!(measure(13, "test"), 0);
        assert_eq!(ascent(13), None);
    }

    #[test]
    fn a_missing_glyph_draws_a_block_rather_than_vanishing() {
        // Glif `■` jest w atlasie jako zastepczy. Znaku spoza zakresu nie ma
        // czym narysowac, wiec musi dostac wypelniony prostokat - inaczej
        // rysownik zobaczylby napis bez litery i nie mialby skad wiedziec,
        // co sie stalo. To najgorszy z mozliwych objawow.
        let (w, h) = (200u32, 60u32);
        let mut buf = vec![0u32; (w * h) as usize];
        let unknown = "\u{4E2D}"; // 中 - poza zakresem atlasu

        assert_eq!(
            measure(RAIL_SIZE, unknown),
            measure(RAIL_SIZE, "\u{25A0}"),
            "znak nieznany musi zajmowac tyle, co glif zastepczy"
        );

        draw_clipped(
            &mut buf, w, h, 10, 5, RAIL_SIZE, unknown, 0xFF101018, 64, Align::Left,
        );
        assert!(
            buf.iter().any(|p| *p != 0),
            "znak spoza atlasu narysowal zero pikseli - powinien byc widoczny blok"
        );
    }

    #[test]
    fn a_real_glyph_is_not_replaced_by_the_fallback() {
        // Odwrotnosc poprzedniego: łaczeń i polskie litery sa w atlasie, wiec
        // musza uzywac wlasnych glifow, a nie zamiennika.
        for ch in "ąćęłńóśźżĄĆĘŁŃÓŚŹŻ".chars() {
            assert_ne!(
                measure(RAIL_SIZE, &ch.to_string()),
                measure(RAIL_SIZE, "\u{25A0}"),
                "znak {ch:?} dostał glif zastepczy zamiast własnego"
            );
        }
    }

    #[test]
    fn broken_utf8_does_not_loop_forever() {
        // `str::chars` zamienia uszkodzona sekwencje na U+FFFD i idzie dalej,
        // ale gdyby `next_char` zwracal ten sam ogon, renderer zapetlil by sie -
        // a to jest najgorszy rodzaj bledu w petli rysujacej.
        //
        // `b"a\xC3"` jako literał nie przejdzie: kompilator potrafi udowodnić
        // jego niepoprawność i odrzuca `from_utf8_unchecked` z takim argumentem
        // wglądem bledow. Dlatego bajty powstaja dopiero w trakcie testu - tym
        // samym dekorator nie ma czego sprawdzic.
        let mut bytes = vec![b'a', 0xC3];
        // SAFETY: intencjalnie niepoprawny UTF-8, `0xC3` to pierwszy bajt
        // dwubajtowej sekwencji bez drugiego. Test nie czyta tego jako tekstu -
        // sprawdza wylacznie zachowanie dekodera.
        let broken = unsafe { std::str::from_utf8_unchecked(bytes.as_slice()) };

        let (ch, rest) = next_char(broken).expect("niepusty napis daje znak");
        assert_eq!(ch, 'a', "poprawny znak przed uszkodzeniem musi przejsc");

        let (ch, rest) = next_char(rest).expect("uszkodzony bajt to nadal znak");
        assert_eq!(ch, '\u{FFFD}', "uszkodzony bajt daje substytut");
        assert!(rest.is_empty(), "ogon musi sie skrocic, inaczej pętla nie kończy");
    }

    #[test]
    fn drawing_never_touches_pixels_outside_the_box() {
        // Bufor z wartownikiem: gdyby rysowanie wyszlo o jeden piksel za
        // pole, indeks poza `len` dalby panike, a test to zglosi. To jest
        // jedyny test, ktory lapie indeksowanie w petli po pikselach.
        let (w, h) = (200u32, 60u32);
        let mut buf = vec![0u32; (w * h) as usize];
        draw_clipped(
            &mut buf, w, h, 10, 5, RAIL_SIZE, "obszary robocze oraz więcej aplikacji",
            0xFF101018, RAIL_BOX, Align::Left,
        );
    }

    #[test]
    fn drawing_off_the_left_edge_is_clipped_not_a_panic() {
        // Panel i szyna nigdy nie wysadzaja, ale debugowanie rysunku z ujemna
        // wspolrzedna jest typowe, a panika na `u32` jest gorsza od braku
        // rysunku.
        let (w, h) = (200u32, 60u32);
        let mut buf = vec![0u32; (w * h) as usize];
        draw_clipped(&mut buf, w, h, -50, -20, RAIL_SIZE, "test", 0xFF101018, 64, Align::Left);
    }
}
