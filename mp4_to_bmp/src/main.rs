// mp4_to_bmp — rozkłada plik mp4 na surowe bitmapy (.bmp, 24-bit, BEZ
// kompresji — RLE ani ZIP, ani nic, sam nagłówek + piksele).
//
// Rust sam z siebie nie potrafi zdekodować H.264/H.265 (to byłby osobny,
// wielomiesięczny projekt), więc do samego dekodowania wideo używamy
// zainstalowanego w systemie `ffmpeg`/`ffprobe` jako zewnętrznego procesu —
// ffmpeg dekoduje klatki do surowego strumienia BGR24 na stdout, a ten
// program tylko tnie ten strumień na klatki i owija każdą w minimalny
// nagłówek BMP. Żadnych crate'ów, żadnego kodowania obrazu — czysty std.
//
// Wymagania: `ffmpeg` i `ffprobe` muszą być w PATH (apt install ffmpeg).
//
// Użycie:
//   mp4_to_bmp <wejscie.mp4> [katalog_wyjsciowy]
//
// Wynik: katalog_wyjsciowy/frame_000001.bmp, frame_000002.bmp, ...

use std::env;
use std::fs;
use std::io::{BufReader, Read, Write};
use std::process::{Command, Stdio};

fn main() {
    if let Err(e) = run() {
        eprintln!("blad: {e}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        return Err(format!(
            "uzycie: {} <wejscie.mp4> [katalog_wyjsciowy]",
            args.get(0).map(String::as_str).unwrap_or("mp4_to_bmp")
        ));
    }
    let input = &args[1];
    let out_dir = args.get(2).cloned().unwrap_or_else(|| "frames".to_string());

    check_tool("ffmpeg")?;
    check_tool("ffprobe")?;

    if !std::path::Path::new(input).is_file() {
        return Err(format!("nie znaleziono pliku wejsciowego: {input}"));
    }

    let (width, height) = probe_resolution(input)?;
    println!("wejscie: {input}  ({width}x{height})");

    fs::create_dir_all(&out_dir).map_err(|e| format!("nie mozna utworzyc {out_dir}: {e}"))?;

    // -f rawvideo -pix_fmt bgr24: strumień czystych bajtów B,G,R na piksel,
    // bez żadnego kontenera ani kompresji — to samo co ma trafić do BMP,
    // więc nie trzeba potem konwertować kolejności kanałów.
    let mut child = Command::new("ffmpeg")
        .args([
            "-v", "error",
            "-i", input,
            "-f", "rawvideo",
            "-pix_fmt", "bgr24",
            "-",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("nie udalo sie uruchomic ffmpeg: {e}"))?;

    let stdout = child.stdout.take().ok_or("brak stdout z ffmpeg")?;
    let mut reader = BufReader::new(stdout);

    let row_bytes = width * 3;
    let frame_bytes = row_bytes * height;
    let mut frame_buf = vec![0u8; frame_bytes];

    let mut frame_no: u64 = 0;

    loop {
        match read_exact_or_eof(&mut reader, &mut frame_buf)? {
            false => break, // koniec strumienia (EOF na granicy klatki)
            true => {
                frame_no += 1;
                let path = format!("{out_dir}/frame_{frame_no:06}.bmp");
                write_bmp(&path, &frame_buf, width, height)
                    .map_err(|e| format!("nie udalo sie zapisac {path}: {e}"))?;

                if frame_no % 50 == 0 {
                    println!("zapisano {frame_no} klatek...");
                }
            }
        }
    }

    let status = child.wait().map_err(|e| format!("blad ffmpeg: {e}"))?;
    if !status.success() && frame_no == 0 {
        return Err("ffmpeg zakonczyl sie bledem i nie wyprodukowal zadnej klatki".to_string());
    }

    println!("gotowe: {frame_no} klatek w '{out_dir}/'");
    Ok(())
}

fn check_tool(name: &str) -> Result<(), String> {
    Command::new(name)
        .arg("-version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_err(|_| format!("'{name}' nie jest zainstalowany / nie ma go w PATH"))?;
    Ok(())
}

/// Pobiera szerokosc i wysokosc pierwszego strumienia wideo przez ffprobe.
fn probe_resolution(input: &str) -> Result<(usize, usize), String> {
    let out = Command::new("ffprobe")
        .args([
            "-v", "error",
            "-select_streams", "v:0",
            "-show_entries", "stream=width,height",
            "-of", "csv=s=x:p=0",
            input,
        ])
        .output()
        .map_err(|e| format!("nie udalo sie uruchomic ffprobe: {e}"))?;

    if !out.status.success() {
        return Err(format!(
            "ffprobe zwrocil blad: {}",
            String::from_utf8_lossy(&out.stderr)
        ));
    }

    let text = String::from_utf8_lossy(&out.stdout);
    let text = text.trim();
    let (w, h) = text
        .split_once('x')
        .ok_or_else(|| format!("nie rozumiem wyjscia ffprobe: '{text}'"))?;

    let width: usize = w.trim().parse().map_err(|_| "zla szerokosc z ffprobe".to_string())?;
    let height: usize = h.trim().parse().map_err(|_| "zla wysokosc z ffprobe".to_string())?;

    if width == 0 || height == 0 {
        return Err("ffprobe zwrocil zerowa rozdzielczosc".to_string());
    }

    Ok((width, height))
}

/// Czyta dokladnie `buf.len()` bajtow. Zwraca Ok(true) jesli sie udalo,
/// Ok(false) jesli strumien skonczyl sie dokladnie na granicy klatki
/// (koniec pliku), Err jesli urwal sie w polowie klatki (uszkodzony strumien).
fn read_exact_or_eof<R: Read>(reader: &mut R, buf: &mut [u8]) -> Result<bool, String> {
    let mut filled = 0;
    while filled < buf.len() {
        let n = reader
            .read(&mut buf[filled..])
            .map_err(|e| format!("blad odczytu strumienia klatek: {e}"))?;
        if n == 0 {
            if filled == 0 {
                return Ok(false); // czysty koniec strumienia
            }
            return Err("strumien urwal sie w polowie klatki".to_string());
        }
        filled += n;
    }
    Ok(true)
}

/// Zapisuje surowa, nieskompresowana bitmape 24-bit BGR (BITMAPFILEHEADER +
/// BITMAPINFOHEADER + piksele). `pixels` to width*height*3 bajtow w kolejnosci
/// B,G,R, wiersz po wierszu, gora->dol (uzywamy ujemnej wysokosci w naglowku,
/// zeby BMP tez czytal je jako top-down — bez potrzeby odwracania wierszy).
fn write_bmp(path: &str, pixels: &[u8], width: usize, height: usize) -> std::io::Result<()> {
    let row_unpadded = width * 3;
    let padding = (4 - (row_unpadded % 4)) % 4;
    let row_padded = row_unpadded + padding;
    let image_size = row_padded * height;
    let data_offset: u32 = 14 + 40;
    let file_size: u32 = data_offset + image_size as u32;

    let mut f = fs::File::create(path)?;

    // BITMAPFILEHEADER (14 bajtow)
    f.write_all(b"BM")?;
    f.write_all(&file_size.to_le_bytes())?;
    f.write_all(&0u16.to_le_bytes())?; // reserved1
    f.write_all(&0u16.to_le_bytes())?; // reserved2
    f.write_all(&data_offset.to_le_bytes())?;

    // BITMAPINFOHEADER (40 bajtow)
    f.write_all(&40u32.to_le_bytes())?; // rozmiar naglowka info
    f.write_all(&(width as i32).to_le_bytes())?;
    f.write_all(&(-(height as i64) as i32).to_le_bytes())?; // ujemne = top-down
    f.write_all(&1u16.to_le_bytes())?; // planes
    f.write_all(&24u16.to_le_bytes())?; // bits per pixel
    f.write_all(&0u32.to_le_bytes())?; // compression = BI_RGB (brak)
    f.write_all(&(image_size as u32).to_le_bytes())?;
    f.write_all(&2835i32.to_le_bytes())?; // ~72 DPI, bez znaczenia
    f.write_all(&2835i32.to_le_bytes())?;
    f.write_all(&0u32.to_le_bytes())?; // colors used
    f.write_all(&0u32.to_le_bytes())?; // colors important

    let zeros = [0u8; 3];
    for row in 0..height {
        let start = row * row_unpadded;
        f.write_all(&pixels[start..start + row_unpadded])?;
        if padding > 0 {
            f.write_all(&zeros[..padding])?;
        }
    }

    Ok(())
}
