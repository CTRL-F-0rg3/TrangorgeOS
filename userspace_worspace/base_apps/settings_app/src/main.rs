//! Menu ustawień wyświetlania - dwa warianty za przełącznikiem.
//!
//! # Po co dwa
//!
//! Wariant literalny odtwarza zrzut ekranu: 78-px szyna, 821 px wysokości,
//! pięć uciętych etykiet. Wariant zwarty ma tę samą szerokość, ale 244 px
//! wysokości i etykiety, które da się przeczytać. Oba rysują to samo menu, więc
//! porównanie w jednym uruchomieniu mówi więcej niż opis.
//!
//! # Uruchomienie
//!
//! ```sh
//! cargo run -p settings-app                      # wszystkie cztery stany
//! cargo run -p settings-app -- --variant compact # tylko wariant zwarty
//! ```
//!
//! Na hoście zapisuje pliki PPM. Na prawdziwym systemie ten sam kod rysuje w
//! buforze pobranym z jądra przez `uspace::kernel::KernelClient`; różni się
//! wyłącznie tym, skąd bierze się `&mut [u32]` - patrz `present()`.

mod dock;
mod dock_compact;
mod theme;

use std::io::Write;

/// Który wariant rysować.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Variant {
    Literal,
    Compact,
}

impl Variant {
    fn parse(name: &str) -> Option<Self> {
        match name {
            "literal" => Some(Self::Literal),
            "compact" => Some(Self::Compact),
            _ => None,
        }
    }
}

/// Zapisuje bufor jako PPM (P6).
///
/// Ten sam format co `demo_gfx`: plik, który daje się otworzyć w dowolnym
/// edytorze i porównać z rzutem. Na systemie zamiast tego bufor trafia do
/// framebuffera, więc ta funkcja jest wyłącznie drogą diagnostyczną.
fn write_ppm(path: &str, fb: &[u32], w: u32, h: u32) -> std::io::Result<()> {
    let mut file = std::fs::File::create(path)?;
    write!(file, "P6\n{w} {h}\n255\n")?;

    let mut data = Vec::with_capacity((w * h * 3) as usize);
    for px in fb {
        data.push(((px >> 16) & 0xFF) as u8);
        data.push(((px >> 8) & 0xFF) as u8);
        data.push((px & 0xFF) as u8);
    }
    file.write_all(&data)
}

/// Rysuje jeden wariant w jednym stanie i zapisuje wynik.
///
/// Osobna funkcja, bo stan zamknięty i otwarty rysuje inaczej: w wariancie
/// literalnym otwarty zjada kółko powiadomień, w zwartym nie.
fn render(variant: Variant, open: bool) -> (Vec<u32>, String) {
    let (w, h) = (theme::CANVAS_W, theme::CANVAS_H);
    let mut buf = vec![0u32; (w * h) as usize];

    theme::fill_background(&mut buf);

    let name = match (variant, open) {
        (Variant::Literal, false) => "settings_literal.ppm",
        (Variant::Literal, true) => "settings_literal_open.ppm",
        (Variant::Compact, false) => "settings_compact.ppm",
        (Variant::Compact, true) => "settings_compact_open.ppm",
    };

    match variant {
        Variant::Literal => {
            if open {
                dock::draw_open(&mut buf);
            } else {
                dock::draw_closed(&mut buf);
            }
        }
        Variant::Compact => {
            if open {
                dock_compact::draw_open(&mut buf);
            } else {
                dock_compact::draw_closed(&mut buf);
            }
        }
    }

    (buf, name.to_string())
}

fn main() {
    // `--variant` wybiera jeden wariant; bez niego rysujemy oba. Domyślnie
    // rysujemy oba, bo porównanie w jednym uruchomieniu jest powodem, dla
    // którego ten plik istnieje.
    //
    // Nierozpoznane opcje są BŁĘDEM, nie cichym pominięciem: `--varant=compact`
    // (literówka) bez tej kontroli wyrenderowałby oba warianty, a użytkownik
    // uznałby, że dostał to, o co prosił.
    let mut requested = None;
    for arg in std::env::args().skip(1) {
        if let Some(name) = arg.strip_prefix("--variant=") {
            requested = Some(match Variant::parse(name) {
                Some(v) => v,
                None => {
                    eprintln!("[settings] nieznany wariant: {name}");
                    eprintln!("[settings] dostepne: literal, compact");
                    std::process::exit(2);
                }
            });
        } else if arg.starts_with('-') {
            eprintln!("[settings] nieznana opcja: {arg}");
            eprintln!("[settings] uzycie: --variant=literal|compact");
            std::process::exit(2);
        }
    }

    let variants: Vec<Variant> = match requested {
        Some(v) => vec![v],
        None => vec![Variant::Literal, Variant::Compact],
    };

    for variant in variants {
        for open in [false, true] {
            let (buf, name) = render(variant, open);
            match write_ppm(&name, &buf, theme::CANVAS_W, theme::CANVAS_H) {
                Ok(()) => println!(
                    "[settings] {} — {}x{}, {}",
                    name,
                    theme::CANVAS_W,
                    theme::CANVAS_H,
                    if open { "otwarty" } else { "zamknięty" }
                ),
                Err(error) => eprintln!("[settings] nie udało się zapisać {name}: {error}"),
            }
        }
    }

    // Podsumowanie geometrii, bo to jedyne miejsce, w którym widać, czy
    // wariant zwarty faktycznie jest mniejszy.
    println!(
        "[settings] szyna: wariant literalny {} px wysokości, wariant zwarty {} px",
        literal_rail_height(),
        dock_compact::rail_height()
    );
}

/// Wysokość zajmowana przez kolumnę w wariancie literalnym, w pikselach.
///
/// Liczona od góry E1 do dołu paska zegara: 22 do 843, czyli 821 px na
/// płótnie 856. Zegar wchodzi tu mimo że jest poza szyną, bo w tym układzie
/// jest ostatnim elementem pionowym i bez niego porównanie wysokości byłoby
/// nieuczciwe - wariant zwarty ma na dole dokładnie ten sam pasek.
fn literal_rail_height() -> u32 {
    (dock::E5_Y + dock::E5_H as i32 - dock::E1_Y) as u32
}
