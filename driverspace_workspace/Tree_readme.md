driverspace_workspace/
├── Cargo.toml                     # Główny workspace manifest
├── rust-toolchain.toml            # Wymuszenie nightly i specyficznych targetów
│
├── lib/                           # 🧰 FUNDAMENT: API, INFRASTRUKTURA I FRAMEWORKI
│   │
│   ├── kapi-abi/                  # Nowe API Jądra: Definicje binarne (Single Source of Truth)
│   │   ├── Cargo.toml
│   │   ├── build.rs               # Generowanie nagłówków C/Odin przez cbindgen
│   │   └── src/
│   │       ├── lib.rs             # Tylko re-eksporty i flagi no_std
│   │       ├── primitives.rs      # Podstawowe typy (Handle, CapId, Status)
│   │       ├── opcodes.rs         # Enumy z numerami wywołań (Syscall/IPC opcodes)
│   │       ├── errors.rs          # Wspólne kody błędów (DsError)
│   │       ├── capabilities.rs    # Struktury uprawnień (Capability tokens)
│   │       ├── wire/              # Serializacja/Deserializacja wiadomości
│   │       │   ├── mod.rs
│   │       │   ├── encode.rs      # Zapis do bufora
│   │       │   ├── decode.rs      # Odczyt z bufora
│   │       │   └── endian.rs      # Konwersje kolejności bajtów
│   │       └── payloads/          # Konkretne struktury wiadomości (#[repr(C)])
│   │           ├── mod.rs
│   │           ├── mem.rs         # Żądania mapowania pamięci
│   │           ├── ipc.rs         # Żądania tworzenia kanałów
│   │           ├── dev.rs         # Żądania rejestracji urządzeń
│   │           └── irq.rs         # Żądania obsługi przerwań
│   │
│   ├── kapi-syscall/              # Surowe wywołania systemowe (SVC / INT)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── arch/              # Implementacje zależne od architektury
│   │       │   ├── mod.rs
│   │       │   ├── x86_64.rs      # Makra asm! dla x86_64 (syscall/sysenter)
│   │       │   └── riscv64.rs     # Makra asm! dla RISC-V (ecall)
│   │       └── wrappers.rs        # Bezpieczne (lub unsafe) wrappery nad surowym asm
│   │
│   ├── ds-ipc/                    # Infrastruktura IPC (Komunikacja między procesami)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── channel.rs         # Abstrakcja kanału (dwukierunkowego)
│   │       ├── port.rs            # Abstrakcja portu (nasłuchiwanie)
│   │       ├── router.rs          # Logika routowania wiadomości
│   │       ├── shared_mem.rs      # Zarządzanie buforami pamięci współdzielonej
│   │       └── sync/              # Primitivy synchronizacji dla IPC
│   │           ├── mod.rs
│   │           ├── spinlock.rs    # Spinlock (no_std)
│   │           └── semaphore.rs   # Semafory dla kolejek wiadomości
│   │
│   ├── ds-mem/                    # Zarządzanie pamięcią w przestrzeni sterownika
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── alloc/             # Własne alokatory (brak std::alloc)
│   │       │   ├── mod.rs
│   │       │   ├── buddy.rs       # Alokator Buddy (dla dużych bloków)
│   │       │   ├── slab.rs        # Alokator Slab (dla małych, stałych obiektów)
│   │       │   └── heap.rs        # Globalny heap sterownika
│   │       ├── dma/               # Obsługa DMA (Direct Memory Access)
│   │       │   ├── mod.rs
│   │       │   ├── buffer.rs      # Bufory DMA (pamięć fizyczna/ciągła)
│   │       │   └── mapping.rs     # Mapowanie pamięci dla urządzeń
│   │       └── vmm.rs             # Abstrakcje nad Wirtualną Pamięcią (stronicowanie)
│   │
│   ├── ds-log/                    # System logowania
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── levels.rs          # Enumy poziomów (Trace, Info, Error, Panic)
│   │       ├── formatter.rs       # Formatowanie tekstu (bez std::fmt)
│   │       ├── transport.rs       # Wysyłanie logów przez IPC do managera
│   │       └── macros.rs          # Makra: ds_log::info!, ds_log::error!
│   │
│   ├── ds-fw-block/               # Framework dla urządzeń blokowych (Dyski, USB Mass)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── traits.rs          # Trait BlockDevice (read, write, flush)
│   │       ├── request.rs         # Struktura żądania I/O (LBA, rozmiar, bufor)
│   │       ├── queue.rs           # Kolejka żądań (I/O Scheduler)
│   │       └── partition.rs       # Abstrakcja tablic partycji (MBR/GPT)
│   │
│   ├── ds-fw-audio/               # Framework dla Audio
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── traits.rs          # Trait AudioDevice (play, record)
│   │       ├── stream.rs          # Abstrakcja strumienia audio
│   │       ├── formats.rs         # Formaty próbek (PCM, Float, Sample rates)
│   │       └── ringbuffer.rs      # Lock-free ring buffer dla danych audio
│   │
│   ├── ds-fw-gpu/                 # Framework dla GPU / Display
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── traits.rs          # Trait DisplayDevice, GpuDevice
│   │       ├── framebuffer.rs     # Abstrakcja bufora ramki (pitch, format)
│   │       ├── modeset.rs         # Ustawianie rozdzielczości i trybu
│   │       ├── edid.rs            # Parser EDID (dane monitora)
│   │       └── cmd_buffer.rs      # Abstrakcja bufora poleceń GPU (Command Submission)
│   │
│   └── ds-fw-input/               # Framework dla HID (Klawiatura, Mysz, Wacom)
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs
│           ├── traits.rs          # Trait InputDevice (poll_event)
│           ├── events.rs          # Typy zdarzeń (KeyPress, MouseMove, PenPressure)
│           ├── scancodes.rs       # Mapowanie scancode'ów na kody klawiszy
│           └── hid_parser.rs      # Parser raportów HID (USB HID descriptors)
│
├── crates/                        # 🧠 ZARZĄDZANIE: DAEMONY DRIVERSPACE
│   │
│   ├── ds-manager/                # Główny daemon zarządzający sterownikami
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs            # Entry point, inicjalizacja pętli zdarzeń
│   │       ├── lifecycle/         # Zarządzanie cyklem życia sterowników
│   │       │   ├── mod.rs
│   │       │   ├── spawn.rs       # Ładowanie i startowanie procesów sterowników
│   │       │   ├── monitor.rs     # Watchdog (restartowanie craszy sterowników)
│   │       │   └── shutdown.rs    # Bezpieczne zamykanie
│   │       ├── resources/         # Przydzielanie zasobów sprzętowych
│   │       │   ├── mod.rs
│   │       │   ├── irq.rs         # Routing i przydzielanie IRQ
│   │       │   ├── mmio.rs        # Mapowanie pamięci urządzeń
│   │       │   └── dma.rs         # Przydzielanie buforów DMA
│   │       ├── ipc_handler.rs     # Nasłuchiwanie żądań od jądra i sterowników
│   │       └── config.rs          # Wczytywanie konfiguracji (np. z TOML/JSON)
│   │
│   └── ds-registry/               # Rejestr urządzeń i sterowników
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs
│           ├── database.rs        # Przechowywanie mapy DeviceID -> Driver
│           ├── matching.rs        # Logika dopasowywania (PCI ID, USB ID, ACPI)
│           └── state.rs           # Śledzenie stanu urządzeń (Podłączone, Usunięte, Błąd)
│
└── drivers/                       # 🚗 STEROWNIKI SPRZĘTOWE
    │
    ├── amdgpu_driver/             # Sterownik AMD GPU
    │   ├── Cargo.toml
    │   ├── build.rs               # (Opcjonalnie) kompilacja C jeśli używasz blobów
    │   └── src/
    │       ├── main.rs            # Entry point, inicjalizacja ds-runtime
    │       ├── probe.rs           # Detekcja karty, odczyt BAR-ów
    │       ├── mmio.rs            # Wrappery do odczytu/zapisu rejestrów MMIO
    │       ├── irq.rs             # Obsługa przerwań GPU
    │       ├── registers/         # Definicje rejestrów (atomowo!)
    │       │   ├── mod.rs
    │       │   ├── gc.rs          # Graphics Controller
    │       │   ├── sdma.rs        # System DMA
    │       │   └── nbio.rs        # Northbridge IO
    │       ├── ring_buffer.rs     # Implementacja pierścieni poleceń (Ring Buffers)
    │       └── display/           # Podmoduł wyświetlania
    │           ├── mod.rs
    │           └── phy.rs         # Inicjalizacja PHY (Physical layer)
    │
    ├── audiodriver/               # Sterownik Audio (Hybryda Rust/C/Okładka na Odina)
    │   ├── Cargo.toml
    │   ├── build.rs               # Kompilacja kodu C i linkowanie z Odinem
    │   ├── c/                     # Kod C (jeśli potrzebny do specyficznych DSP)
    │   │   ├── dsp.c
    │   │   └── dsp.h
    │   └── src/
    │       ├── main.rs
    │       ├── codec.rs           # Komunikacja z kodekiem audio (I2C/HDA)
    │       ├── dma_stream.rs      # Zarządzanie strumieniami DMA dla audio
    │       └── ffi.rs             # Funkcje `extern "C"` wywoływane przez C/Okładki
    │
    ├── intelgpu_driver/           # Sterownik Intel GPU (i915/xe)
    │   ├── Cargo.toml
    │   └── src/
    │       ├── main.rs
    │       ├── probe.rs
    │       ├── gt.rs              # Graphics Technology (rdzeń GPU)
    │       ├── display.rs         # Display Engine (DE)
    │       └── guc.rs             # GuC (Graphics Microcontroller) firmware loading
    │
    ├── netcam_driver/             # Sterownik kamer sieciowych (UVC/USB)
    │   ├── Cargo.toml
    │   └── src/
    │       ├── main.rs
    │       ├── uvc.rs             # USB Video Class protocol parser
    │       ├── isoc.rs            # Obsługa transferów izochronicznych USB
    │       └── controls.rs        # Panel sterowania (jasność, kontrast, zoom)
    │
    ├── vgpu/                      # Wirtualne GPU (Głównie C, z Rust FFI)
    │   ├── Cargo.toml
    │   ├── build.rs               # Kompilacja vgpu_driver.c
    │   ├── c/
    │   │   ├── vgpu_driver.c      # Główna logika w C
    │   │   ├── vgpu.h
    │   │   └── vgpu_mmio.c        # Operacje na pamięci w C
    │   └── src/
    │       ├── lib.rs             # Rust wrapper, eksportujący ABI dla ds-manager
    │       └── ffi.rs             # Definicje `extern "C"` dla funkcji z C
    │
    └── wacomgraphic_driver/       # Tablet graficzny Wacom (HID)
        ├── Cargo.toml
        └── src/
            ├── main.rs
            ├── hid_report.rs      # Parsowanie specyficznych raportów Wacom
            ├── pressure.rs        # Konwersja surowych danych na nacisk
            └── tilt.rs            # Obsługa nachylenia rysika (Tilt X/Y)