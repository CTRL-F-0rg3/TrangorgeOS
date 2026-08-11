# TrangorgeOS - kernel

**Wersja:** 0.0.4
**Bazuje na:** [Writing an OS in Rust](https://os.phil-opp.com/) (Phil Opperman), rozdziały: *A Minimal Kernel* + *VGA Text Mode* + *CPU Exceptions* + *Double Faults* + *Hardware Interrupts* + *Introduction to Paging* + *Paging Implementation*

Minimalny kernel x86_64 w Rust, bootowany przez crate `bootloader = "0.9"`, z własnym targetem, printami przez VGA text mode i własnym mini-frameworkiem do selftestów modułów.

---

## 1. Wymagania

```bash
rustup install nightly
rustup override set nightly          # w folderze kernel/
rustup component add rust-src --toolchain nightly
rustup component add llvm-tools-preview --toolchain nightly

cargo install bootimage
```

QEMU:
```bash
# Arch/Manjaro
sudo pacman -S qemu-full
# Debian/Ubuntu
sudo apt install qemu-system-x86
```

---

## 2. Budowanie i uruchamianie

```bash
cargo run
```

Zbuduje kernel, złoży bootowalny obraz (`bootimage`) i odpali go w QEMU (dzięki `runner = "bootimage runner"` w `.cargo/config.toml`).

Ręcznie, krok po kroku:
```bash
cargo bootimage
qemu-system-x86_64 -drive format=raw,file=target/x86_64-kernel/debug/bootimage-kernel.bin
```

Przydatne flagi QEMU:
- `-serial stdio` - output z portu szeregowego do terminala
- `-display curses` - gdy nie ma GUI/X11
- `-no-reboot -no-shutdown` - QEMU nie resetuje się w kółko po panice/triple faulcie

---

## 3. Struktura projektu

```
kernel/
├── Cargo.toml
├── Cargo.lock
├── .cargo/
│   └── config.toml        # ustawienia builda (unstable flags, target, runner)
├── x86_64-kernel.json      # wlasny target spec (zamiast wbudowanego x86_64-unknown-none)
├── src/
│   ├── main.rs             # punkt wejscia (_start), init(), hlt_loop(), rejestr testow
│   ├── gdt.rs               # GDT + TSS, oddzielny stos dla double fault (IST)
│   ├── interrupts.rs         # IDT: wyjatki CPU (breakpoint, double fault, page fault) + PIC (timer, klawiatura)
│   ├── vga_buffer.rs         # sterownik VGA text mode + print!/println!/print_colored!
│   └── testing.rs            # framework selftestow modulow
├── ctrlfile.toml            # config osobnego narzedzia budujacego (zig/pacman) - nie dotyczy Rusta
└── ctrlfilemaker            # skrypt dla powyzszego narzedzia - nie dotyczy Rusta
```

---

## 4. Własny target (`x86_64-kernel.json`)

Zamiast wbudowanego `x86_64-unknown-none` używamy własnego pliku target spec, bo:
- chcemy pełną kontrolę nad ABI kernela (soft-float, brak red zone, itd.)
- pozwala to na dalsze modyfikacje bez czekania na zmiany we wbudowanym targecie

Kluczowe pola:
```json
"panic-strategy": "abort",
"relocation-model": "static",
"disable-redzone": true,
"features": "-mmx,-sse,+soft-float"
```

UWAGA: **`relocation-model: static` jest krytyczne.** Bez tego nowsze nightly linkują kernel jako pozycyjnie-niezależny (PIC), co tworzy dodatkowy segment ELF pod adresem `0x0` i bootloader wywala się z `PageAlreadyMapped` przy próbie zmapowania kernela.

Jeśli kiedyś trzeba wygenerować plik od zera (np. po zmianie wersji nightly i błędach o nieznanych/złych polach w JSON-ie), najbezpieczniej wygenerować bazę bezpośrednio z własnego rustc, a potem dopisać tylko potrzebne nadpisania:

```bash
rustc -Z unstable-options --print target-spec-json --target x86_64-unknown-none > x86_64-kernel.json
```
- i dopisać do niego: `panic-strategy`, `relocation-model`, `disable-redzone`, `features`.

### `.cargo/config.toml`

```toml
[unstable]
json-target-spec = true                              # wymagane od cargo 1.95+ do użycia .json targetu
build-std-features = ["compiler-builtins-mem"]
build-std = ["core", "compiler_builtins"]

[build]
target = "x86_64-kernel.json"

[target.'cfg(target_os = "none")']
runner = "bootimage runner"
```

---

## 5. VGA text mode (`vga_buffer.rs`)

Sterownik pisze bezpośrednio do pamięci VGA pod adresem `0xb8000` (25 wierszy × 80 kolumn, każdy znak to bajt ASCII + bajt koloru).

Globalny writer (`WRITER`, `lazy_static` + `spin::Mutex`) pozwala pisać z dowolnego miejsca w kernelu przez makra:

```rust
println!("Hello {}", "world");   // domyślny kolor (żółty na czarnym)
print!("bez nowej linii");
print_colored!(Color::LightRed, "błąd: {}", kod);   // dowolny kolor, jednorazowo
```

Dostępne kolory: `Color::{Black, Blue, Green, Cyan, Red, Magenta, Brown, LightGray, DarkGray, LightBlue, LightGreen, LightCyan, LightRed, Pink, Yellow, White}`.

Automatyczne przewijanie ekranu (`new_line()`) działa - po przepełnieniu ostatniego wiersza wszystko jedzie w górę o jeden wiersz.

---

## 6. Framework selftestów (`testing.rs`)

Cel: każdy moduł kernela deklaruje własny test na samej górze swojego pliku, kernel odpala wszystkie testy przy starcie i drukuje kolorowy raport - podobnie do selftestów jądra Linuksa.

### Jak dodać test do modułu

Na **początku pliku modułu** (np. `src/twoj_modul.rs`):

```rust
crate::test_module!({
    // tu logika testu
    if cos_nie_tak {
        return Err("opis błędu / kod błędu");
    }
    Ok("krótki opis co przetestowano")
});
```

To generuje w module publiczną funkcję `pub fn self_test() -> testing::TestResult`.

### Jak zarejestrować test w kernelu

W `src/main.rs`, w tablicy `TESTS`:

```rust
static TESTS: &[Test] = &[
    Test { module: "vga_buffer", func: vga_buffer::self_test },
    Test { module: "twoj_modul", func: twoj_modul::self_test },  // dopisz kolejną linię
];
```

Pamiętaj o `mod twoj_modul;` na górze `main.rs`.

### Format wyniku

```
Running N module test(s)...
<nazwa_modułu (magenta)> <tekst z modułu> [OK (zielony)]
<nazwa_modułu (magenta)> [FAILED (czerwony)] <komunikat błędu>
```

`testing::run_all(TESTS)` jest wołane jako pierwsza rzecz w `_start()`, zanim kernel robi cokolwiek innego.

### Typ wyniku testu

```rust
pub type TestResult = Result<&'static str, &'static str>;
```
- `Ok(opis)` - test przeszedł, `opis` to dodatkowy tekst wypisywany obok nazwy modułu
- `Err(komunikat_błędu)` - test nie przeszedł, treść trafia po `[FAILED]`

UWAGA: bez `alloc` nie ma dynamicznych stringów - komunikaty muszą być stałymi `&'static str` (nie da się np. wstawić dynamicznie obliczonego indeksu bez formatowania na heapie, którego jeszcze nie mamy).

---

## 7. GDT, IDT, PIC, hlt_loop (stabilność + ciepło procesora)

### GDT + TSS (`gdt.rs`)

Osobny stos (Interrupt Stack Table, indeks `DOUBLE_FAULT_IST_INDEX = 0`) dla obsługi double fault. Bez tego, jeśli kernel dostanie double fault w momencie gdy jego normalny stos jest już uszkodzony (np. stack overflow), CPU nie ma gdzie zapisać ramki przerwania i leci w triple fault (czyli twardy reset całego systemu, bez żadnego komunikatu). Z osobnym stosem dostajemy normalny panic z komunikatem zamiast cichego restartu.

`gdt::init()` ładuje GDT, ustawia rejestr CS i ładuje TSS.

### IDT (`interrupts.rs`)

Obsłużone wyjątki CPU:
- `breakpoint` (`int3`) - drukuje ramkę stosu i wraca do działania normalnie
- `double_fault` - panikuje z komunikatem (używając osobnego stosu z GDT)
- `page_fault` - drukuje adres, który spowodował błąd, kod błędu i ramkę stosu, potem `hlt_loop()`

Do tego dwa przerwania sprzętowe przez PIC 8259:
- `Timer` (IRQ0) - na razie tylko potwierdza przerwanie (EOI), przyda się pod przyszły scheduler
- `Keyboard` (IRQ1) - czyta skankod z portu `0x60` i wypisuje go na ekran (surowy skankod, bez tłumaczenia na znaki - to osobny temat na potem)

`interrupts::init_idt()` ładuje IDT.

### Kolejność inicjalizacji

W `main.rs`, funkcja `init()` musi być wywołana **przed** czymkolwiek innym w `_start()`:

```rust
pub fn init() {
    gdt::init();
    interrupts::init_idt();
    unsafe { interrupts::PICS.lock().initialize() };
    x86_64::instructions::interrupts::enable();
}
```

Kolejność ma znaczenie: najpierw GDT (bo IDT double faulta odwołuje się do stosu z GDT), potem IDT, potem inicjalizacja PIC-a (remapuje IRQ na wektory 32-47, żeby nie kolidowały z wyjątkami CPU 0-31), na końcu dopiero `sti` (globalne włączenie przerwań).

### hlt_loop - dlaczego laptop mniej się grzeje

Stary `loop {}` to busy-loop: rdzeń non-stop wykonuje instrukcję "skocz do tego samego miejsca", czyli pracuje na 100% mocy bez przerwy, nawet gdy kernel nic nie robi. `hlt_loop()`:

```rust
pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}
```

używa instrukcji `hlt`, która usypia rdzeń aż do następnego przerwania (np. timer). Realnie zmniejsza to zużycie mocy/ciepło generowane przez rdzeń w QEMU/na sprzęcie, bo CPU nie robi nic zamiast robić nic-ale-szybko. Używany teraz zarówno w `_start()` po zakończeniu testów, jak i w panic handlerze.

### Test modułu `interrupts`

Na górze `interrupts.rs` jest test wywołujący programowo przerwanie breakpoint (`int3`) i sprawdzający, że kernel nie pada - jeśli IDT nie jest poprawnie załadowany, ten test (a właściwie cały kernel) skończy się triple faultem/resetem zamiast ładnego `[FAILED]`, więc brak komunikatu w konsoli też jest sygnałem błędu.

---

## 8. src/memory/ - struktura modulu pamieci

```
src/memory/
├── mod.rs             -- PHYSICAL_MEMORY_OFFSET, globalny MAPPER, globalny FRAME_ALLOCATOR, init()
├── paging.rs           -- active_level_4_table, OffsetPageTable, manualny translate_addr (4 poziomy)
├── frame_allocator.rs   -- BootInfoFrameAllocator (na bazie boot_info.memory_map), EmptyFrameAllocator
└── mapping.rs            -- realne mapowanie strony: allocate_frame + map_to + zapis/odczyt + unmap
```

`memory::init(physical_memory_offset, memory_map)` wywolywane raz w `kernel_main()`, zaraz po `init()` (GDT/IDT/PIC) a przed testami:
1. zapisuje offset pamieci fizycznej
2. buduje `OffsetPageTable` (bezpieczny wrapper x86_64 na manualne chodzenie po tablicach) i chowa go w `MAPPER`
3. buduje `BootInfoFrameAllocator` z prawdziwej mapy pamieci od bootloadera i chowa go w `FRAME_ALLOCATOR`

Oba globalne stany sa za `spin::Mutex`, wiec dostep jest bezpieczny nawet gdy pozniej dojda przerwania/wiele rdzeni siegajace po pamiec rownolegle.

### Dlaczego trzy pliki, nie jeden

- `paging.rs` odpowiada wylacznie za CZYTANIE istniejacych tablic stron (translate_addr, dostep do poziomu 4)
- `frame_allocator.rs` odpowiada wylacznie za PRZYDZIAL nowej pamieci fizycznej
- `mapping.rs` odpowiada za ZMIANE mapowan (laczy oba powyzsze, bo map_to potrzebuje frame allocatora do budowy brakujacych tablic posrednich)

Rozdzielenie odpowiedzialnosci: kod czytajacy nie wie nic o alokacji, kod alokujacy nic nie wie o mapowaniu, kod mapujacy korzysta z obu przez ich publiczne API - zaden z tych plikow nie musi znac szczegolow implementacji pozostalych.

### Co kazdy test faktycznie sprawdza

- `memory::paging` - realny odczyt poziomu 4 (>0 aktywnych wpisow) + manualny walk 4 poziomow przez `translate_addr` porownany z faktycznym adresem fizycznym VGA (0xb8000)
- `memory::frame_allocator` - 8 kolejnych alokacji zwraca 8 unikalnych, wyrownanych do 4KiB ramek (bez duplikatow)
- `memory::mapping` - najmocniejszy test: alokuje realna ramke, mapuje na nia swiezo wybrana strone wirtualna, zapisuje przez wskaznik konkretna wartosc, czyta z powrotem przez ten sam wskaznik, odmapowuje. Jesli to przejdzie, paging realnie dziala od poczatku do konca - nie tylko "nie crashuje"

## 9. Co dalej (TODO / kolejne rozdziały bloga)

- [x] CPU Exceptions - IDT, breakpoint, page fault
- [x] Double Faults - bezpieczny stos dla double fault (IST, GDT)
- [x] Hardware Interrupts - PIC, timer, klawiatura
- [x] hlt_loop zamiast busy-loop (mniejsze zużycie mocy/ciepło)
- [x] Przejście z reczengo `_start` na `bootloader::entry_point!` + `BootInfo` (feature `map_physical_memory`) - wymagane, żeby w ogóle mieć dostęp do offsetu pamięci fizycznej i mapy pamięci od bootloadera
- [x] Paging: `active_level_4_table`, `OffsetPageTable`, manualny `translate_addr` po wszystkich 4 poziomach
- [x] Frame allocator na bazie `boot_info.memory_map` (`BootInfoFrameAllocator`)
- [x] Realne mapowanie strony (alokacja + `map_to` + zapis/odczyt + `unmap`) z testem end-to-end
- [ ] Testing w prawdziwym `cargo test` (harness z `bootimage test-runner`, wyjście przez `isa-debug-exit`) - do rozważenia jako uzupełnienie własnego `testing.rs`
- [ ] Heap allocation (`alloc`, custom allocator - np. bump albo linked-list) - `BootInfoFrameAllocator` jest teraz gotowy jako fundament pod to
- [ ] Wielordzeniowość (SMP): parsowanie ACPI/MADT (crate `acpi`), inicjalizacja Local APIC (zamiast starego PIC 8259 dla timera), wybudzanie AP-ów (INIT-SIPI-SIPI), per-core stosy i GDT/TSS, atomowy scheduler/lock-free struktury. Wymaga dzialającego heap - to dopiero nastepny duży etap po punkcie powyzej.

---

## 10. Znane pułapki (żeby nie tracić czasu drugi raz)

1. **`.json` target wymaga `json-target-spec = true`** w `.cargo/config.toml` (cargo 1.95+, styczeń 2026) - bez tego: `error: .json target specs require -Zjson-target-spec`.
2. **Pola w target JSON bywają typowane inaczej niż w starych tutorialach** - np. `target-pointer-width` i `target-c-int-width` muszą być liczbami (`64`, `32`), nie stringami (`"64"`) w nowszych nightly.
3. **`executables` (liczba mnoga), nie `executable`** - literówka, która daje `unknown field` przy ładowaniu target specu.
4. **`relocation-model: static` musi być ustawione jawnie** - patrz sekcja 4, inaczej `PageAlreadyMapped` przy starcie w QEMU.
5. **`compiler-builtins-mem`, nie `compiler-build-mem`** - literówka w `build-std-features` wywala build ze skrajnie niejasnym błędem.
6. **`extern "x86-interrupt"` wymaga `#![feature(abi_x86_interrupt)]` na górze `main.rs`** - to niestabilna cecha kompilatora (dostępna tylko na nightly, którego i tak używamy). Bez tego atrybutu handlery przerwań w `interrupts.rs` się nie skompilują.
7. **Kolejność `init()` ma znaczenie** - GDT musi być załadowany przed IDT (double fault odwołuje się do stosu z GDT), a `sti` (włączenie przerwań) musi być na samym końcu, po zainicjalizowaniu PIC-a - inaczej przerwanie sprzętowe przyjdzie zanim IDT/PIC są gotowe i kernel się wywali.
8. **Crate `x86_64` w wersji 0.15 zmienił kilka nazw API względem 0.14** - `GlobalDescriptorTable::add_entry` to teraz `append`, a `InterruptDescriptorTable` indeksuje się po `u8`, nie po `usize`. Jeśli update crate w `Cargo.toml`, zawsze usuń `Cargo.lock` i zrób `cargo clean`, żeby nie zostały dwie wersje tej samej biblioteki w grafie zależności.
9. **Niestabilny trait `Step` w bibliotece standardowej potrafi się zmienić z nightla na nightla** - starsze wersje `x86_64` (np. 0.14.13) mogą przestać się kompilować z błędem `missing forward_overflowing, backward_overflowing`. Rozwiązanie: aktualna wersja `x86_64` (0.15.x), nie próba łatania samego triku ze Step.
10. **`bootloader::entry_point!` wymaga feature `map_physical_memory`** w `Cargo.toml` (`bootloader = { version = "0.9", features = ["map_physical_memory"] }`), inaczej `boot_info.physical_memory_offset` będzie zawsze zerem i każda próba odczytu tablic stron przez ten offset skończy się page faultem.
