use super::font::FONT8X8;
use super::framebuffer::{Framebuffer, rgb};
use super::galaxy;
use crate::mm::ffi;

// Rozmiar glifu w pikselach. Renderujemy ZAWSZE 1:1 — bez przeskalowania,
// bez łączenia bitów. Dzięki temu znak nigdy nie jest ściśnięty ani
// zdeformowany, niezależnie od rozdzielczości.
const GLYPH_W: usize = 8;
const GLYPH_H: usize = 8;

// Maksymalny rozmiar bufora tekstowego, jaki dostarcza `vga_buffer`
// (klasyczny układ 80x25). Siatka konsoli nigdy nie wykroczy poza te
// granice — przy niższych rozdzielczościach będzie mniejsza (mniej
// widocznych kolumn/wierszy), ale zawsze 1:1 z fontem.
pub const MAX_COLS: usize = 80;
pub const MAX_ROWS: usize = 25;

const VGA_PALETTE: [(u32, u32, u32); 16] = [
    (0,0,0),(0,0,170),(0,170,0),(0,170,170),
    (170,0,0),(170,0,170),(170,85,0),(170,170,170),
    (85,85,85),(85,85,255),(85,255,85),(85,255,255),
    (255,85,85),(255,85,255),(255,255,85),(255,255,255),
];

static mut FB: Option<Framebuffer> = None;
static mut CLEAN: *mut u8 = core::ptr::null_mut();

// Faktyczna siatka tekstu, wyliczana w init() na podstawie rozdzielczości
// (width/GLYPH_W, height/GLYPH_H, capowane do MAX_COLS/MAX_ROWS).
static mut COLS: usize = 0;
static mut ROWS: usize = 0;

// Cache ostatnio narysowanej zawartości każdej komórki (znak, atrybut).
// refresh() przerysowuje tylko komórki, które faktycznie się zmieniły —
// to główna oszczędność, bo pełne przejście 80x25 z rysowaniem glifu
// piksel po pikselu jest kosztowne, a w praktyce większość ekranu się
// nie zmienia między klatkami.
static mut CELL_CACHE: [(u8, u8); MAX_COLS * MAX_ROWS] = [(0, 0); MAX_COLS * MAX_ROWS];
static mut CACHE_VALID: bool = false;

// ŁATKA, nie fix: coś w vga_buffer (poza tymi plikami) serwuje znaki w
// linii w odwrotnej kolejności względem widocznej szerokości konsoli.
// Zamiast czekać na naprawę u źródła, czytamy kolumnę `col` jako
// `cols-1-col`. Jeśli/gdy prawdziwa przyczyna zostanie znaleziona i
// naprawiona w vga_buffer.rs, to trzeba wyłączyć (ustawić na false),
// inaczej tekst znów będzie odwrócony, tylko w drugą stronę.
const REVERSE_TEXT_COLS: bool = true;

fn fb() -> &'static mut Framebuffer {
    unsafe { FB.as_mut().unwrap() }
}

/// Aktualna liczba kolumn siatki tekstu (zależna od rozdzielczości).
pub fn cols() -> usize {
    unsafe { COLS }
}

/// Aktualna liczba wierszy siatki tekstu (zależna od rozdzielczości).
pub fn rows() -> usize {
    unsafe { ROWS }
}

fn set_palette_rgb332() {
    use x86_64::instructions::port::Port;

    let mut idx = Port::<u8>::new(0x3C8);
    let mut data = Port::<u8>::new(0x3C9);

    unsafe {
        idx.write(0);

        for i in 0..256u32 {
            let r = ((i >> 5) & 0x7) * 9;
            let g = ((i >> 2) & 0x7) * 9;
            let b = (i & 0x3) * 21;

            data.write(r as u8);
            data.write(g as u8);
            data.write(b as u8);
        }
    }
}

// Kursor sprzętowy trybu tekstowego (rejestr CRTC 0x0A, bit 5 = disable)
// zostaje "żywy" po przełączeniu w tryb graficzny, jeśli nikt go jawnie
// nie wyłączy — stąd migający/kwadratowy artefakt na środku ekranu.
fn disable_text_cursor() {
    use x86_64::instructions::port::Port;

    let mut idx = Port::<u8>::new(0x3D4);
    let mut data = Port::<u8>::new(0x3D5);

    unsafe {
        idx.write(0x0Au8);
        data.write(0x20u8);
    }
}

fn delay() {
    for _ in 0..50_000 {
        core::hint::spin_loop();
    }
}

pub fn test_fill(r: u32, g: u32, b: u32) {
    let (w, h) = {
        let f = fb();
        (f.width, f.height)
    };

    for y in 0..h {
        for x in 0..w {
            fb().set(x, y, rgb(r, g, b));
        }
    }
}

pub fn init(fb_addr: u64, width: u32, height: u32, stride: u32) -> bool {
    if width == 0 || height == 0 || stride == 0 {
        return false;
    }

    if !unsafe { ffi::mm_ready() } {
        return false;
    }

    let width = width as usize;
    let height = height as usize;
    let stride = stride as usize; // 8 bpp: bajty na wiersz

    // Zaokrąglamy do strony (vmm_map_device wymaga wyrównania; okno VGA to 64 KiB).
    let size = ((stride * height) + 0xFFF) & !0xFFF;

    let ptr = if fb_addr >= 0xFFFF800000000000 {
        fb_addr as *mut u8
    } else {
        let mut virt = 0u64;

        if !unsafe { ffi::vmm_map_device(fb_addr, size, &mut virt) } {
            return false;
        }

        virt as *mut u8
    };

    set_palette_rgb332();
    disable_text_cursor();

    // Framebuffer VGA 13h pod 0xA0000 jest natywnie top-down (wiersz 0 =
    // góra ekranu). Wymuszamy FLIP=false tutaj, bo jeśli coś gdzie indziej
    // w kernelu ustawiło ten globalny flag na true (np. zaszłość po innym
    // trybie), cały ekran — tło i tekst — renderuje się odwrócony w pionie.
    //
    // FLIP_X=true to próba na podstawie objawu z ostatniego zrzutu ekranu
    // (tekst poprawny w pionie, lustrzany w poziomie). JEŚLI PO TEJ ZMIANIE
    // dalej jest źle (albo zrobi się gorzej — np. galaktyka wygląda ok, ale
    // tekst nadal lustrzany, lub litery są teraz w dobrą stronę ale kolejność
    // słów/kolumn w linii jest zła), ustaw to z powrotem na false i wyślij
    // mi zawartość vga_buffer::text_cell() — wtedy odbicie siedzi w źródle
    // danych tekstu, nie w warstwie graficznej, i trzeba naprawić tam.
    //
    // Jeśli kiedyś dojdzie tryb z naprawdę innym framebufferem (np. VESA
    // LFB), obie flagi lepiej przekazywać jako parametr zamiast nadpisywać
    // na sztywno tutaj.
    unsafe {
        super::framebuffer::FLIP = false;
        super::framebuffer::FLIP_X = true;
    }

    unsafe {
        FB = Some(Framebuffer {
            ptr,
            width,
            height,
            stride,
        });

        // Siatka konsoli dopasowana do rozdzielczości: tyle kolumn/wierszy,
        // ile zmieści się glifami 8x8 bez ściskania, capowane do rozmiaru
        // bufora tekstowego. Przy 320x200 wyjdzie 40x25 (nie 80x25) —
        // mniej widocznych kolumn, ale bez deformacji fontu.
        COLS = (width / GLYPH_W).clamp(1, MAX_COLS);
        ROWS = (height / GLYPH_H).clamp(1, MAX_ROWS);

        // Nowa rozdzielczość = cache nieaktualny, wymuszamy pełny redraw.
        CACHE_VALID = false;
    }

    // Liczba klatek fade-inu skalowana w dół dla dużych rozdzielczości —
    // przy 320x200 (64k px) 17 klatek jest tanie, ale przy większych
    // trybach ten sam koszt liczony per-piksel narasta szybko. Zamiast
    // ciąć jakość, po prostu robimy mniej kroków fade-inu.
    let total_px = width * height;
    let steps: u32 = if total_px > 400_000 {
        4
    } else if total_px > 150_000 {
        8
    } else {
        17
    };
    let step_size = (256 / steps.max(1)).max(1);

    let mut t = 0u32;
    loop {
        galaxy::render(fb(), t.min(256));
        delay();
        if t >= 256 {
            break;
        }
        t += step_size;
    }

    let mut buf = 0u64;

    if !unsafe { ffi::vmm_alloc(size, 1, &mut buf) } {
        return false;
    }

    unsafe {
        CLEAN = buf as *mut u8;

        core::ptr::copy_nonoverlapping(
            fb().ptr as *const u8,
            CLEAN,
            fb().stride * fb().height,
        );
    }

    refresh();

    true
}

pub fn refresh() {
    if unsafe { FB.is_none() } {
        return;
    }

    // Pierwsze odświeżenie po init()/zmianie rozdzielczości: pełny
    // redraw. Kolejne wywołania: tylko zmienione komórki.
    let first = unsafe { !CACHE_VALID };

    if first {
        let (h, s) = {
            let f = fb();
            (f.height, f.stride)
        };

        unsafe {
            core::ptr::copy_nonoverlapping(CLEAN, fb().ptr, s * h);
        }
    }

    let (cols, rows) = unsafe { (COLS, ROWS) };

    for row in 0..rows {
        for col in 0..cols {
            let src_col = if REVERSE_TEXT_COLS { cols - 1 - col } else { col };
            let (ch, attr) = crate::vga_buffer::text_cell(row, src_col);
            let idx = row * MAX_COLS + col;

            let changed = unsafe { CELL_CACHE[idx] != (ch, attr) };

            if !first && !changed {
                continue;
            }

            unsafe {
                CELL_CACHE[idx] = (ch, attr);
            }

            draw_cell(row, col, ch, attr);
        }
    }

    unsafe {
        CACHE_VALID = true;
    }
}

/// Rysuje jedną komórkę tekstu 1:1 z fontem 8x8 — bez przeskalowania,
/// bez łączenia bitów. Bit 7 = najbardziej wysunięty w lewo piksel
/// wiersza `gy` (kolejność: góra->dół, lewo->prawo), więc znak nigdy
/// nie wychodzi odbity ani zniekształcony.
///
/// Tło komórki: jeśli atrybut ma bg == 0 (domyślny/czarny — tak wygląda
/// zdecydowana większość bufora tekstowego), NIE malujemy płaskiego
/// koloru, tylko przywracamy piksel spod tekstu z bufora CLEAN (czyli
/// mgławica prześwituje zza tekstu, tak jak było zamierzone). Jeśli bg
/// jest jawnie ustawione na coś innego niż 0 (np. podświetlenie), maluje
/// się kryjąco tym kolorem — jak wcześniej.
fn draw_cell(row: usize, col: usize, ch: u8, attr: u8) {
    let bg_idx = attr >> 4;
    let transparent_bg = bg_idx == 0;

    let fg = VGA_PALETTE[(attr & 0x0F) as usize];
    let bg = VGA_PALETTE[bg_idx as usize];

    let glyph = if (0x20..=0x7E).contains(&ch) {
        FONT8X8[(ch - 0x20) as usize]
    } else {
        FONT8X8[('?' as u8 - 0x20) as usize]
    };

    let (w, h) = {
        let f = fb();
        (f.width, f.height)
    };

    for gy in 0..GLYPH_H {
        let bits = glyph[gy];
        let py = row * GLYPH_H + gy;

        if py >= h {
            continue;
        }

        for gx in 0..GLYPH_W {
            let px = col * GLYPH_W + gx;

            if px >= w {
                continue;
            }

            let lit = bits & (0x80 >> gx) != 0;

            if lit {
                fb().set(px, py, rgb(fg.0, fg.1, fg.2));
            } else if transparent_bg {
                // Kopiujemy surowy bajt indeksu palety wprost z CLEAN —
                // taniej niż get()+set() (bez konwersji RGB w obie strony)
                // i dokładnie odtwarza to, co tam narysowała galaxy::render.
                // offset() uwzględnia FLIP/FLIP_X, więc to zostaje spójne
                // niezależnie od orientacji framebuffera.
                unsafe {
                    let off = fb().offset(px, py);
                    let idx_byte = *CLEAN.add(off);
                    *fb().ptr.add(off) = idx_byte;
                }
            } else {
                fb().set(px, py, rgb(bg.0, bg.1, bg.2));
            }
        }
    }
}