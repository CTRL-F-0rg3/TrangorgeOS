#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""Generator atlasu bitmapowego dla warstwy tekstowej TrangorgeOS.

Powód, dla którego to w ogóle istnieje
--------------------------------------
`allde` i `uspace::gfx::render` mają font 8x8 o zakresie ASCII 32..126. Nie
potrafi on narysować ani polskich znaków diakrytycznych (`draw_text` pomija
bajty spoza zakresu, wiec `ó` znika cicho), ani rozmiaru 17-19 px. To menu
wymaga obu, a dorzucanie rasteryzera TTF do runtime'u byloby dokladnie tym,
czego ten projekt nie chce: zaleznosci, floatow i pamieci.

Rozwiazaniem jest atlas: raz zrasteryzowany, zapisany jako plik binarny i
czytany przy starcie. Runtime robi tylko blend pikseli.

Metryki pochodza z `hmtx` (dokladne szerokosci z fontu), a maski z FreeType
przez Pillow. Rozmiar jest ustalany tutaj, nie w runtime - dzieki temu nie ma
zadnego kodu rasteryzujacego ani w jadrze, ani w aplikacji.

Uzycie
------
    python3 tools/genfont.py [--ttf PATH] [--out DIR]

Domyślnie szuka DejaVuSans w typowych lokalizacjach. Wynik ląduje w
`uspace/src/gfx/font_atlas.rs` (metryki) oraz `uspace/src/gfx/font_atlas.bin`
(maski), oba w repo - zbudowanie systemu nie wymaga Pythona ani fontu.
"""

import argparse
import sys
from pathlib import Path

try:
    from fontTools.ttLib import TTFont
except ImportError:
    sys.exit("brak fontTools: pip install fonttools")

try:
    from PIL import Image, ImageDraw, ImageFont
except ImportError:
    sys.exit("brak Pillow: pip install pillow")


# Rozmiary, ktore faktycznie wystepuja w interfejsie. Dodanie czwartego
# kosztuje ok. 18 KB w repo, wiec lista jest krotka celowo.
SIZES = (14, 17, 19)

# Glif zastępczy dla znakow, ktorych krój nie ma: rysowany jako wypełniony
# prostokąt. Renderowanie go tak, jakby byl poprawny, ukrylo by blad w
# pokryciu znakow; widoczny blok mowi "brak glifu".
FALLBACK = "■"


def build_codepoints():
    """Zbiera unikalny, posortowany zbior znakow do wytworzenia.

    Sortowanie po kodepoincie, nie wg napisow: inaczej kolejność zależy od
    tego, jak zbudowano listy zrodlowe, a atlas ma byc powtarzalny.
    """
    chars = set()

    # ASCII drukowalny.
    for cp in range(0x20, 0x7F):
        chars.add(chr(cp))

    # Latin-1 Supplement: wiekszosc akcentow i symboli.
    for cp in range(0xA0, 0x100):
        chars.add(chr(cp))

    # Polskie znaki spoza Latin-1.
    chars.update("ąćęłńóśźżĄĆĘŁŃÓŚŹŻ")

    # Typografia polska: myślniki, trzy kropki, cudzyslowy, apostrofy.
    chars.update("–—…„“”‘’")

    chars.add(FALLBACK)
    return sorted(chars, key=ord)


CANDIDATE_TTF = [
    "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
    "/usr/share/fonts/dejavu/DejaVuSans.ttf",
    "/usr/share/fonts/TTF/DejaVuSans.ttf",
    "/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf",
    "/usr/share/fonts/truetype/liberation2/LiberationSans-Regular.ttf",
]


def find_ttf(explicit=None):
    """Znajdzie plik kroja: podany wprost, w typowych miejscach, gdziekolwiek."""
    if explicit:
        path = Path(explicit)
        if path.is_file():
            return path
        sys.exit(f"nie ma takiego pliku: {explicit}")

    for candidate in CANDIDATE_TTF:
        path = Path(candidate)
        if path.is_file():
            return path

    # Ostatnia deska ratunku: jakikolwiek bezszeryfowy z katalogu fontow.
    for root in ("/usr/share/fonts", str(Path.home() / ".local/share/fonts")):
        base = Path(root)
        if not base.is_dir():
            continue
        for found in sorted(base.rglob("*.ttf")):
            name = found.name.lower()
            if "dejavusans.ttf" in name or "liberationsans-regular" in name:
                return found

    sys.exit("nie znaleziono bezszeryfowego TTF; podaj sciezke: --ttf /sciezka")


class Rasterizer:
    """Rasteryzuje pojedyncze znaki w Pillow, z poprawnym osadzeniem baseline."""

    def __init__(self, ttf: Path):
        self.path = ttf
        self.tt = TTFont(str(ttf), fontNumber=0)
        self.upm = self.tt["head"].unitsPerEm
        self.hmtx = self.tt["hmtx"]
        self.cmap = self.tt.getBestCmap()
        # Pillow chce sciezke, a `truetype` parsuje plik od nowa przy kazdym
        # wywolaniu, wiec fonty ladujemy raz na rozmiar.
        self._fonts = {}

    def has(self, ch: str) -> bool:
        return ord(ch) in self.cmap

    def _font(self, size: int):
        if size not in self._fonts:
            self._fonts[size] = ImageFont.truetype(str(self.path), size)
        return self._fonts[size]

    def advance(self, ch: str, size: int) -> int:
        """Szerokosc posuwnicza w pikselach, zaokraglona tak jak robi FreeType.

        Zaokraglenie idzie w dol: 17.6 px -> 17. Dlatego w calym ukladzie
        suma zaokraglenych szerokosci jest mniejsza niz suma dokladnych, a
        ostatni znak wiersza nie wychodzi o piksel za szeroko.
        """
        name = self.cmap.get(ord(ch))
        if name is None:
            name = self.cmap.get(ord(FALLBACK))
        if name is None:
            return 0
        return int(round(self.hmtx[name][0] * size / self.upm))

    def bitmap(self, ch: str, size: int):
        """Zwraca (maska, bearing_x, bearing_y, width, height).

        `bearing_y` jest liczone od baseline w gore, czyli dodatnie. Znak bez
        maski - spacja - zwraca `(None, 0, 0, 0, 0)`.
        """
        font = self._font(size)
        ascent, _descent = font.getmetrics()

        # Rezerwa z obu stron na glify wychodzace poza pole. Robimy jeden
        # obraz na znak; `getbbox` zwraca pusty prostokat dla spacji.
        box = size * 3
        img = Image.new("L", (box, box), 0)
        draw = ImageDraw.Draw(img)
        # Domyslna kotwica Pillow to "la" (lewo, linia wstepujaca), wiec
        # y=0 to gora linii, a baseline lezy dokladnie w `ascent`.
        draw.text((box // 2, 0), ch, font=font, fill=255)

        bbox = img.getbbox()
        if bbox is None:
            return None, 0, 0, 0, 0

        x0, y0, x1, y1 = bbox
        mask = img.crop(bbox)
        bearing_x = x0 - (box // 2)
        bearing_y = ascent - y0
        return mask, bearing_x, bearing_y, x1 - x0, y1 - y0


def pack_4bpp(mask) -> bytes:
    """Pakuje 8-bitowa maske do 4 bitow na piksel, dwa piksele w bajcie.

    16 poziomow alpha wystarcza dla tekstu na jednokolorowym tle, a plik jest
    o polowe mniejszy. Jedyna strata to 16 poziomow na pojedynczym pikselu,
    czego na 17-19 px nie da sie dostrzec.
    """
    data = mask.tobytes()
    out = bytearray((len(data) + 1) // 2)
    for i, value in enumerate(data):
        nibble = value >> 4  # 0..15
        if i & 1:
            out[i >> 1] |= nibble
        else:
            out[i >> 1] = nibble << 4
    return bytes(out)


def main() -> int:
    parser = argparse.ArgumentParser(description="Generator atlasu fontu")
    parser.add_argument("--ttf", help="sciezka do pliku TTF")
    parser.add_argument(
        "--out",
        default="uspace/src/gfx",
        help="katalog na font_atlas.rs i font_atlas.bin",
    )
    args = parser.parse_args()

    ttf = find_ttf(args.ttf)
    print(f"[genfont] krój: {ttf}")

    ras = Rasterizer(ttf)
    chars = build_codepoints()
    covered = sum(1 for c in chars if ras.has(c))
    print(f"[genfont] znaków: {len(chars)} (obecnych w kroju: {covered})")

    out_dir = Path(args.out)
    out_dir.mkdir(parents=True, exist_ok=True)

    bitmaps = bytearray()
    # (size, codepoint, advance, bearing_x, bearing_y, w, h, offset, length)
    metrics = []

    for size in SIZES:
        for ch in chars:
            advance = ras.advance(ch, size)
            mask, bx, by, w, h = ras.bitmap(ch, size)

            if mask is None:  # spacja i znaki bez maski
                metrics.append((size, ord(ch), advance, 0, 0, 0, 0, 0, 0))
                continue

            packed = pack_4bpp(mask)
            offset = len(bitmaps)
            bitmaps.extend(packed)
            metrics.append((size, ord(ch), advance, bx, by, w, h, offset, len(packed)))

    (out_dir / "font_atlas.bin").write_bytes(bytes(bitmaps))

    # Wysokosci wstepujaca i zejscia z `hhea`, w tym samym pliku liniowym, ktorego
    # uzyly FreeType przy rasteryzacji. `OS/2.sTypo*` jest zgodne tylko dla
    # czcionek bez inkrementu i z czcionek linearnych, a DejaVu ma oba, wiec
    # mieszanie plikow dalo by inna bazowa niz ta, ktora zrodlo faktycznie
    # renderuje.
    hhea = ras.tt["hhea"]
    ascent = {s: int(round(hhea.ascender * s / ras.upm)) for s in SIZES}
    descent = {s: int(round(-hhea.descender * s / ras.upm)) for s in SIZES}

    emit_rust(
        out_dir / "font_atlas.rs", chars, metrics, ttf, len(bitmaps), ascent, descent
    )

    print(f"[genfont] maski  : font_atlas.bin ({len(bitmaps):,} B)")
    print("[genfont] metryki: font_atlas.rs")
    print(f"[genfont] wpisów : {len(metrics)}")
    return 0


def emit_rust(path: Path, chars, metrics, ttf: Path, total: int, ascent, descent) -> None:
    """Wypisuje plik Rust z metrykami. Maski siedza obok, w .bin."""
    by_size = {s: [] for s in SIZES}
    for row in metrics:
        by_size[row[0]].append(row)

    lines = []
    add = lines.append

    add("//! Atlas bitmapowy - METRYKI. Wygenerowane przez `tools/genfont.py`.")
    add("//!")
    add("//! Ten plik jest generowany. Ręczne zmiany zostaną nadpisane przy")
    add("//! ponownym uruchomieniu generatora; zmieniaj generator, nie wynik.")
    add("//!")
    add(f"//! Krój: `{ttf.name}`")
    add(f"//! Glify: {len(chars)} · rozmiary: {', '.join(str(s) for s in SIZES)} px")
    add("//! Maski: `font_atlas.bin`, ładowane przez `include_bytes!`.")
    add("//!")
    add("//! Znak spoza zakresu renderuje się jako wypełniony prostokąt,")
    add("//! zamiast znikać - brak glifu ma być widoczny, nie cichy.")
    add("")
    add("/// Bity maski na piksel.")
    add("pub const BPP: u8 = 4;")
    add("")
    add("/// Liczba bajtow w `font_atlas.bin`.")
    add(f"pub const BITMAP_BYTES: usize = {total};")
    add("")
    add("/// Maski alfa, 4 bpp na piksel, dwa piksele w bajcie.")
    add("pub static BITMAPS: &[u8] = include_bytes!(\"font_atlas.bin\");")
    add("")
    add("/// Opis jednego glifu w jednym rozmiarze.")
    add("///")
    add("/// `#[repr(C)]` i `Copy`, bo atlas ma być statycznym zestawem danych,")
    add("/// a nie czymś, co trzeba konstruować przy starcie.")
    add("#[repr(C)]")
    add("#[derive(Clone, Copy, PartialEq, Eq, Debug)]")
    add("pub struct Glyph {")
    add("    /// Punkt kodowy znaku.")
    add("    pub codepoint: u32,")
    add("    /// Szerokość posuwnicza w pikselach - o tyle przesuwa się pismo.")
    add("    ///")
    add("    /// Od niej zależy cały układ wiersza, więc to nie jest szerokość")
    add("    /// obrazka: słowa z szerokimi literami wyglądają normalnie, mimo")
    add("    /// że maska jest węższa.")
    add("    pub advance: u16,")
    add("    /// Przesunięcie maski w prawo od pisowni.")
    add("    pub bearing_x: i8,")
    add("    /// Przesunięcie maski w górę od linii bazowej.")
    add("    pub bearing_y: i8,")
    add("    pub width: u8,")
    add("    pub height: u8,")
    add("    /// Offset w `BITMAPS`; `0` oznacza brak maski, czyli spację.")
    add("    pub offset: u32,")
    add("    /// Długość maski w bajtach.")
    add("    pub length: u16,")
    add("}")
    add("")

    for size in SIZES:
        rows = by_size[size]
        add(f"/// Metryki glifów w {size} px.")
        add(f"pub static GLYPHS_{size}: [Glyph; {len(rows)}] = [")
        for (_sz, cp, adv, bx, by, w, h, off, ln) in rows:
            add(
                f"    Glyph {{ codepoint: {cp}, advance: {adv}, "
                f"bearing_x: {bx}, bearing_y: {by}, width: {w}, "
                f"height: {h}, offset: {off}, length: {ln} }},"
            )
        add("];")
        add("")

    sizes = ", ".join(str(s) for s in SIZES)
    add("/// Wszystkie rozmiary, rosnąco.")
    add(f"pub static ALL_SIZES: [u32; {len(SIZES)}] = [{sizes}];")
    add("")
    add("/// Odległość od gory linii do linii bazowej, w pikselach.")
    add("///")
    add("/// To jest wysokosc wstepujaca kroju, nie wysokosc litery. Rozninica")
    add("/// jest znaczaca: `ó` i `s` nie wchodza do kwadratu wstepujacego, a")
    add("/// mylnie liczona bazowa przesuwa wiersz o wysokosc x.")
    add("///")
    add("/// Plik liniowy na tej wysokosci jest zgodny z `hhea`, a nie z")
    add("/// `OS/2.sTypoAscender`: to `hhea` opisuje linie, ktora faktycznie")
    add("/// renderuje FreeType, i o ten sam plik pytaliismy przy pomiarach.")
    for size in SIZES:
        add(f"pub const ASCENT_{size}: u32 = {ascent[size]};")
    add("")
    add("/// Glebokosc zejscia kroju, w pikselach. Dodatna.")
    add("///")
    add("/// Uzywana przy wyliczaniu wysokosci wiersza, gdyby ktos chcial")
    add("/// policzyc odstep od polowy x-height zamiast od bazowej.")
    for size in SIZES:
        add(f"pub const DESCENT_{size}: u32 = {descent[size]};")
    add("")
    add("/// Tabela glifów dla jednego rozmiaru.")
    add("///")
    add("/// Rozmiar spoza listy to błąd programisty, nie sytuacja do obsługi:")
    add("/// `None`, bo brak rozmiaru oznacza błąd atlasu, a nie stan aplikacji.")
    add("pub fn table_for(size: u32) -> Option<&'static [Glyph]> {")
    add("    match size {")
    for size in SIZES:
        add(f"        {size} => Some(&GLYPHS_{size}),")
    add("        _ => None,")
    add("    }")
    add("}")
    add("")

    path.write_text("\n".join(lines) + "\n", encoding="utf-8")


if __name__ == "__main__":
    sys.exit(main())
