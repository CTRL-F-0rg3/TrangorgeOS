# TrangorgeOS — dokumentacja architektury

> **Status dokumentu:** wersja robocza 0.1  
> **Zakres:** architektura systemu, podział jądra, pamięć i alokator, model sterowników oraz mapa repozytorium  
> **Źródła:** drzewo projektu przekazane przez autora, kod gałęzi `unstable` oraz oficjalna strona projektu [1] [2]

## 1. Cel projektu

TrangorgeOS jest systemem operacyjnym budowanym od podstaw. Projekt ma docelowo obsługiwać kilka architektur procesora: **x86_64** jako pierwszą platformę, a następnie **ARM64** i **RISC-V**. Jądro jest projektowane jako **hybrydowe**, czyli łączy mechanizmy typowe dla jąder monolitycznych z izolowaniem i wydzielaniem części funkcjonalności poza podstawową przestrzeń jądra.

Kod systemu jest wielojęzyczny, ale należy rozróżnić stan bieżący od planów. Publiczna gałąź `unstable` jest raportowana przez GitHub jako przede wszystkim Rust (93,3%), z udziałem Zig (5,9%) i pozostałych języków [1]. Oficjalna strona wymienia Rust jako język jądra, a Ada/SPARK, C i język własny jako elementy planowanego szerszego ekosystemu [2]. W praktyce konkretny język powinien być dobierany do problemu, lecz dokumentacja nie powinna przedstawiać planowanych technologii jako już aktywnych części kernela.

Dokumentacja ma dwa cele. Po pierwsze, ma być **mapą orientacyjną dla autora**, który rozwija wiele podsystemów jednocześnie. Po drugie, ma umożliwiać innym osobom zrozumienie granic odpowiedzialności między jądrem, przestrzenią sterowników, bibliotekami i narzędziami projektu.

## 2. Stan projektu

Według opisu autora jądro jest już zaimplementowane, system plików działa, choć nadal wymaga poprawek, a projekt ma około 22 tysięcy linii kodu samego jądra oraz około 8 tysięcy linii kodu związanego z alokatorem. Sterowniki nie są jeszcze ukończone, ale wiele elementów zostało już wykonanych. Aktualny etap koncentruje się na **driver space**, bibliotekach poza kernelem oraz dopracowaniu jądra przed przejściem jego gałęzi do stabilniejszego stanu.

Poniższa tabela rozdziela informacje potwierdzone przez autora, twierdzenia oficjalnej strony oraz fakty zweryfikowane w kodzie gałęzi `unstable`.

| Obszar | Potwierdzone | Wniosek ze struktury |
|---|---|---|
| Architektura CPU | x86_64, później ARM64 i RISC-V | Obecne artefakty wskazują przede wszystkim na x86_64. |
| Typ jądra | Hybrydowe | Wydzielenie `driverspace` i `driverspacelib` wspiera taki model. |
| Języki | Rust w kernelu; strona wymienia Ada/SPARK, C i język własny jako planowany ekosystem | GitHub raportuje dla bieżącej gałęzi 93,3% Rust, 5,9% Zig i 0,8% innych języków [1]. |
| System plików | Działa, ale ma problemy | W `kernel/src/fs` znajdują się obsługa dysku, MBR, FAT32, ext4, TFS i sterowniki blokowe. |
| Alokator | Około 8 tys. linii według autora | Kod potwierdza stertę opartą obecnie na buddy allocatorze; `HEAP_USE_SLAB` ma wartość `0`, więc slab jest zaimplementowany, ale wyłączony w bieżącej konfiguracji. |
| Sterowniki | Rozwijane, jeszcze nieukończone | Repozytorium zawiera sterowniki GPU, audio, kamery, Wacom, USB, PCI, NIC i VirtIO. |

## 3. Model wysokopoziomowy

System można obecnie opisać jako kilka współpracujących warstw:

```text
+---------------------------------------------------------------+
| Programy, narzędzia i biblioteki użytkowe                    |
| triang-lang, trangorgelibc, mp4_to_bmp, narzędzia ISO        |
+-----------------------------+---------------------------------+
| Driver space                | Biblioteki i kontrakty ABI     |
| driverspace                 | driverspacelib, trangorgelibc  |
| rejestracja i wykonywanie   | syscall, typy, pamięć, I/O     |
| sterowników                 |                                 |
+-----------------------------+---------------------------------+
| Kernel space                | CPU, MM, FS, NIC, USB, PCI,    |
| podstawowe mechanizmy       | przerwania, grafika, terminal  |
+-----------------------------+---------------------------------+
| Sprzęt i firmware           | x86_64; w przyszłości ARM64,   |
|                              | RISC-V                         |
+---------------------------------------------------------------+
```

Diagram jest zgodny z publicznym opisem projektu [2], natomiast szczegóły komunikacji są już częściowo widoczne w implementacji. `kernel/src/driverspaceinit` tworzy osobny `AddressSpace`, alokuje i zeruje dwa pierścienie komunikacyjne oraz stronę `DsInitParams`, po czym mapuje je do driver space z uprawnieniami użytkownika [3]. `service.rs` obsługuje m.in. logowanie, alokację i zwalnianie stron, mapowanie MMIO, odczyt/zapis blokowy oraz operacje audio [4].

## 4. Trójpodział sterowników

Oficjalna strona definiuje trójpodział jako **kernel space**, **driver space** i **user space** [2]. Kod potwierdza, że driver space nie jest wyłącznie nazwą katalogu: kernel przygotowuje dla niego osobną przestrzeń adresową, dwa współdzielone ring buffer’y oraz stronę parametrów inicjalizacyjnych [3]. Na tej podstawie trójpodział można opisać następująco:

1. **Kernel space** — kod, który musi działać z najwyższymi uprawnieniami i bezpośrednim dostępem do mechanizmów jądra. Obejmuje między innymi enumerację PCI i USB, przerwania, DMA, sterowniki blokowe, warstwę sieciową oraz podstawową inicjalizację.
2. **Driver space** — wydzielone środowisko dla sterowników, uruchamiane i obsługiwane przez mechanizmy `driverspaceinit`. Znajdują się tu kontrakty API, inicjalizacja usług oraz osobne pakiety sterowników. Celem tej warstwy jest ograniczenie tego, co każdy sterownik musi wykonywać bezpośrednio w rdzeniu jądra.
3. **User space / warstwa kliencka** — programy i biblioteki korzystające z usług sterowników przez stabilne interfejsy. W repozytorium rolę podstawy tej warstwy pełnią `trangorgelibc`, interfejsy ABI/syscall oraz narzędzia i język `triang-lang`.

### 4.1. Przepływ żądania sterownika

W aktualnym kodzie komunikacja kernel ↔ driver space odbywa się przez dwa pierścienie: `k2d` (kernel-to-driver) i `d2k` (driver-to-kernel). Wiadomość `DsMsg` przenosi identyfikator, komendę, flagi, trzy argumenty oraz status. Wśród zdefiniowanych komend znajdują się między innymi `Init`, `RegisterDriver`, `AttachDevice`, `MapDeviceMemory`, `BindIrq`, `Log`, `AllocPages`, `FreePages`, `MapMmio`, `BlockRead`, `BlockWrite`, `AudioInfo` i `PagePhys` [3] [4].

Przykładowy przepływ powinien wyglądać następująco:

```text
Program lub biblioteka
        |
        v
ABI / syscall / publiczne API
        |
        v
driverspacelib: typy, runtime, logowanie, ring buffer, audio, block
        |
        v
Driver space: API, rejestracja, sesja sterownika, komunikacja
        |
        v
Kernel glue: PCI/USB/DMA/przerwania/pamięć
        |
        v
Kontroler i urządzenie sprzętowe
```

Najważniejsza zasada architektoniczna brzmi: **sterownik nie powinien powielać logiki jądra, a jądro nie powinno znać szczegółów każdego urządzenia**. Kernel powinien dostarczać bezpieczne prymitywy, granicę komunikacyjną, zarządzanie pamięcią, obsługę przerwań i dostęp do magistral. Driver space powinno dostarczać logikę konkretnego urządzenia oraz rejestrować jego możliwości w ustandaryzowany sposób.

### 4.2. Elementy modelu widoczne w repozytorium

| Element | Lokalizacja | Rola |
|---|---|---|
| API driver space | `driverspace/src/api.rs`, `driverspace/src/driversapi` | Publiczne punkty wejścia i API dla przestrzeni sterowników. |
| Biblioteka driver space | `driverspacelib/src` | ABI, runtime, logowanie, pamięć, ring buffer, audio, blok i jack. |
| Inicjalizacja po stronie kernela | `kernel/src/driverspaceinit` | Wejście, komunikacja, komendy inicjalizacyjne, usługi i ABI. |
| Pakiety sterowników | `drivers/*` | Konkretne sterowniki GPU, audio, kamery i urządzeń wskazujących. |
| Interfejsy klienta | `trangorgelibc/src` | Typy ABI, błędy, tabela kernela, syscall oraz warstwy I/O, pamięci i synchronizacji. |

## 5. Architektura jądra

Główny kod jądra znajduje się w `kernel/src`. Z punktu widzenia odpowiedzialności można go podzielić na następujące podsystemy.

| Podsystem | Katalogi / pliki | Odpowiedzialność |
|---|---|---|
| Punkt wejścia i diagnostyka | `main.rs`, `serial.rs`, `vga_buffer.rs`, `testing.rs` | Start systemu, wyjście diagnostyczne, testy i wczesna obserwowalność. |
| ABI | `abi/` | Publiczne kontrakty alokacji, pamięci, systemu plików i API. |
| CPU i przerwania | `cpu/`, `gdt.rs`, `interrupts.rs` | ACPI, LAPIC, GDT, trampoliny, wejścia przerwań i obsługa CPU. |
| Pamięć | `mm/`, `allocator/` | Pamięć fizyczna i wirtualna, mapowanie, stronicowanie, ochrona, cache, procesy oraz sterta. |
| Sterowniki sprzętowe | `drivers/`, `pci.rs` | PCI, USB, urządzenia HID, mass storage, host controllers oraz warstwa glue. |
| System plików | `fs/` | MBR, dyski, ATA PIO, warstwa blokowa, FAT32, ext4, TFS i inicjalizacja FS. |
| Grafika i konsola | `gfx/`, `terminal/`, `vga_buffer.rs` | Framebuffer, VGA, font, konsola, terminal oraz grafika demonstracyjna. |
| Sieć | `nic/` | Ethernet, pakiety, protokoły, urządzenia i VirtIO network queues. |
| Instalacja i aktualizacje | `ctrlinstall/` | Repozytorium, indeksy, manifesty, rozwiązywanie zależności, transakcje i aktualizacje. |
| Driver space | `driverspaceinit/` | Uruchamianie, komunikacja i usługi dla wydzielonej przestrzeni sterowników. |
| Audio | `audio/` | Integracja audio po stronie kernela, w szczególności warstwa jack. |

### 5.1. Zasady zależności

Zależności powinny płynąć od warstw wyższych do niższych przez jawne interfejsy, a nie przez przypadkowe odwołania do implementacji. W praktyce oznacza to, że system plików korzysta z abstrakcji blokowej, sterownik konkretnego dysku implementuje tę abstrakcję, a kod użytkowy komunikuje się z FS przez ABI. Analogicznie, sterownik urządzenia powinien korzystać z usług PCI, USB, DMA, przerwań i pamięci zamiast samodzielnie odtwarzać ich logikę.

Warto traktować katalogi `abi`, `driverspaceinit`, `driverspacelib` i `trangorgelibc` jako **granice architektoniczne**. Każda zmiana w tych obszarach może wpływać na wiele niezależnych komponentów i powinna być opisana w changelogu lub dokumentacji wersji.

## 6. Alokator i podsystem pamięci

Kod `kernel/src/mm` implementuje wielowarstwowy podsystem pamięci, a nie pojedynczą funkcję `alloc`. Rozdzielenie na pamięć fizyczną, wirtualną, stertę, cache, ochronę, DMA i przestrzenie procesów jest właściwe dla systemu, który ma obsługiwać sterowniki oraz izolowane środowiska. Rzeczywista kolejność inicjalizacji w `mm_init()` to: `arch_memory_init` → `paging_init` → `pmm_init` → `vmm_init` → `page_init` → `mapping_init` → `heap_init` → `cache_init` → `paging_subsystem_init` → `isolation_init` → `aspace_subsystem_init` [7].

```text
Warstwy pamięci TrangorgeOS

API Rust / C i mostek FFI
          |
          v
mm/api.rs, mm/ffi.rs, mm/mm_bridge.rs
          |
          +--> pamięć fizyczna: pmm, bitmap, frame
          |
          +--> pamięć wirtualna: vmm, mapping, page, paging, tlb
          |
          +--> sterta: heap, buddy, slab
          |
          +--> pamięć specjalna: contiguous, dma
          |
          +--> cache i lokalność CPU: cache, object_cache, per_cpu
          |
          +--> przestrzenie procesów i ochrona: address_space, mmap,
               guard, isolation, permissions
```

### 6.1. Pamięć fizyczna

Warstwa `mm/alloc/physical` zawiera bitmapę, ramki i PMM. Jej zadaniem jest śledzenie dostępnych jednostek pamięci fizycznej oraz przydzielanie i zwalnianie ramek. Ta warstwa powinna pozostawać możliwie niezależna od typów obiektów używanych przez wyższe warstwy.

### 6.2. Pamięć wirtualna i stronicowanie

Warstwy `virtual`, `paging`, `arch/x86_64` oraz `process` wskazują na rozdzielenie mapowania adresów, tablic stron, TLB i przestrzeni adresowych procesów. W przyszłości jest to naturalne miejsce na wydzielenie kodu zależnego od architektury: mechanizmy ogólne powinny być wspólne, a operacje na tablicach stron, TLB i rejestrach powinny mieć implementacje per-arch.

### 6.3. Sterta, buddy i slab

W bieżącej gałęzi `unstable` aktywną ścieżką sterty jest **buddy allocator**. `heap_init()` zawsze inicjalizuje obszar buddy o rozmiarze 256 MiB, natomiast obsługa slab jest kompilacyjnie wyłączona przez `#define HEAP_USE_SLAB 0`. Po włączeniu tej flagi slab może obsługiwać małe obiekty, ale nie jest to obecna ścieżka domyślna [5].

Kod `buddy.c` potwierdza przydział bloków według rzędu, listy wolnych bloków, mapowanie stron oraz koalescencję z wolnym blokiem-bliźniakiem przy zwalnianiu. `slab.c` zawiera osobną implementację klas obiektów, ale pozostaje obecnie poza domyślną ścieżką wykonania. Publiczne API alokacji wystawia m.in. `kmalloc`, `kzalloc`, `kcalloc`, `krealloc`, `kmalloc_aligned`, `kfree` oraz alokację stron [5] [6].

Praktyczny podział odpowiedzialności powinien być następujący:

| Komponent | Zadanie |
|---|---|
| PMM / bitmapa / frame | Przydział ramek fizycznych. |
| Buddy | Zarządzanie większymi, potęgowymi obszarami pamięci i ewentualna koalescencja bloków. |
| Heap | Wystawienie ogólnego interfejsu alokacji sterty. |
| Slab | Wydajny przydział obiektów o określonym rozmiarze. |
| DMA / contiguous | Przydział obszarów spełniających wymagania sprzętu. |
| Cache / per-CPU | Ograniczenie kosztu blokad i poprawa lokalności. |
| Protection / isolation | Kontrola praw dostępu oraz izolacja przestrzeni. |

### 6.4. Kontrakt, który warto dopisać do kodu

Dokumentacja alokatora powinna w przyszłości odpowiedzieć na sześć pytań: kto jest właścicielem przydzielonej pamięci, jakie są wymagania wyrównania, czy alokacja może blokować, jak zachowuje się system przy braku pamięci, które funkcje są bezpieczne w kontekście przerwania oraz jak pamięć DMA różni się od zwykłej pamięci sterty. Są to kontrakty ważniejsze od samych nazw plików, ponieważ bezpośrednio decydują o poprawności sterowników i systemu plików.

## 7. Sterowniki sprzętowe

Katalog `drivers` zawiera osobne pakiety dla różnych klas urządzeń. Aktualna mapa wygląda następująco.

| Pakiet | Przeznaczenie |
|---|---|
| `amdgpu_driver` | Sterownik GPU AMD. |
| `intelgpu_driver` | Sterownik GPU Intel. |
| `netcam_driver` | Sterownik kamery sieciowej lub urządzenia tego typu. |
| `wacomgraphic_driver` | Sterownik urządzenia graficznego Wacom. |
| `audiodriver` | Sterownik audio; zawiera Rust, integrację jack oraz moduły Odin dla mikrofonu i głośnika. |

W jądrze znajdują się ponadto ogólne elementy obsługi USB i PCI. USB jest podzielone na warstwy core, host controllers EHCI/XHCI oraz klasy urządzeń CDC, HID i mass storage. Taki podział jest korzystny, ponieważ kod wspólny dla wszystkich urządzeń USB nie powinien zależeć od konkretnego kontrolera hosta.

Warstwa sieciowa `kernel/src/nic` zawiera abstrakcje urządzenia i sterownika, Ethernet, pakiety, protokoły oraz implementację VirtIO. Oznacza to, że sieć ma już rozdział między ogólną obsługą pakietów a konkretnym mechanizmem urządzenia.

## 8. Boot, budowanie i narzędzia

Projekt zawiera dwa widoczne warianty elementu startowego: `comgrub` oraz `comlimine`. Pierwszy zawiera `boot.asm`, a drugi pliki C/Rust i nagłówek Limine. Oznacza to, że proces uruchamiania może być rozwijany lub testowany przez więcej niż jedną ścieżkę bootowania.

`iso-builder` jest osobnym narzędziem do tworzenia obrazu ISO. Składa się z modułów Oberonowych (`.Mod`) oraz pliku startowego `Boot`. Skrypty `docker-build.sh`, `docker-run.sh`, `docker-shell.sh` i `Dockerfile` wskazują na przygotowane środowisko kontenerowe do budowania lub uruchamiania projektu.

`targets/x86_64-unknown-none` oraz `kernel/x86_64-kernel.json` wskazują na obecny profil bare-metal dla x86_64. Przy dodawaniu ARM64 i RISC-V należy utrzymać osobne konfiguracje targetu i oddzielić kod zależny od architektury od logiki ogólnej.

`triang-lang` jest osobnym kompilatorem lub językiem projektowym. Zawiera lexer, parser, AST, analizę semantyczną, IR, projekt oraz generatory kodu dla asemblera i C. Katalog `out/x86_64` pokazuje artefakty generowane dla x86_64. `trangorgelibc` dostarcza natomiast bibliotekę z ABI, błędami, syscallami, typami, I/O, pamięcią i synchronizacją.

## 9. Mapa repozytorium

| Katalog | Znaczenie |
|---|---|
| `kernel/` | Główne jądro systemu, jego ABI, pamięć, FS, CPU, sieć i sterowniki. |
| `drivers/` | Osobne pakiety konkretnych sterowników urządzeń. |
| `driverspace/` | Program lub środowisko driver space oraz jego API. |
| `driverspacelib/` | Biblioteka wspólna dla sterowników działających w driver space. |
| `trangorgelibc/` | Biblioteka klienta i warstwa ABI/syscall. |
| `comgrub/` | Wariant komponentu startowego z GRUB. |
| `comlimine/` | Wariant komponentu startowego z Limine. |
| `iso-builder/` | Narzędzie tworzące obraz ISO. |
| `triang-lang/` | Język/kompilator z generatorami C i ASM. |
| `mp4_to_bmp/` | Narzędzie pomocnicze do konwersji MP4 do BMP. |
| `targets/` | Konfiguracje targetów kompilacji. |
| `Dockerfile` i skrypty Docker | Powtarzalne środowisko budowania i uruchamiania. |

## 10. Zasady utrzymania dokumentacji

Każdy nowy podsystem powinien mieć krótki opis odpowiedzialności, publiczne wejście, zależności oraz informację o tym, w której przestrzeni działa. Dla zmian ABI należy zapisywać wersję interfejsu, kompatybilność wsteczną oraz komponenty, które trzeba przebudować.

Warto prowadzić dokumentację w układzie odpowiadającym repozytorium, ale nie kopiować bezpośrednio całego drzewa plików. Drzewo mówi, gdzie coś się znajduje; dokumentacja powinna dodatkowo wyjaśniać **dlaczego** dany moduł istnieje i **jak** komunikuje się z resztą systemu.

Dla sterownika przydatny jest stały szablon:

| Pole | Treść do uzupełnienia |
|---|---|
| Nazwa | Nazwa urządzenia i pakietu. |
| Przestrzeń wykonania | Kernel space, driver space albo user space. |
| Magistrala | PCI, USB, VirtIO lub inna. |
| Zależności | Pamięć, DMA, przerwania, kolejki, ABI. |
| Punkt inicjalizacji | Funkcja lub komenda uruchamiająca sterownik. |
| API | Operacje udostępniane klientom. |
| Ograniczenia | Brakujące funkcje, problemy i założenia sprzętowe. |
| Status | Projektowany, eksperymentalny, działający lub stabilny. |

## 11. Otwarte pytania techniczne

Poniższe kwestie powinny zostać doprecyzowane w następnej wersji dokumentacji, ponieważ nie da się ich rozstrzygnąć na podstawie samej struktury plików:

1. Jakie są docelowe granice uprawnień oraz które funkcje z protokołu driver space są już stabilne?
2. Kod potwierdza osobną przestrzeń adresową, ale trzeba doprecyzować pełny cykl życia procesu/obrazu driver space oraz reakcję na jego awarię.
3. Jaki jest formalny format komunikatów między kernelem a driver space?
4. Jak wygląda cykl życia sterownika: discovery, init, start, stop, reset, unload i obsługa awarii?
5. Czy buddy, slab i heap tworzą jeden łańcuch alokacji, czy są niezależnymi allocatorami dla różnych typów pamięci?
6. Czy `DsMsg` i komendy z `kernel/src/driverspaceinit/abi/abi.rs` mają już gwarantowaną kompatybilność między wersjami?
7. Jakie są minimalne wymagania dla implementacji ARM64 i RISC-V?
8. Które sterowniki są obecnie działające na prawdziwym sprzęcie, a które są dopiero szkieletem?

## 12. Podsumowanie

TrangorgeOS ma już wyraźny podział na jądro, przestrzeń sterowników, biblioteki, pakiety urządzeń, narzędzia bootowania i narzędzia developerskie. Najważniejszym elementem architektury jest rozdzielenie mechanizmów podstawowych — pamięci, przerwań, PCI, USB, DMA, systemu plików i sieci — od kodu konkretnego urządzenia uruchamianego w modelu driver space.

Największą wartością kolejnej iteracji dokumentacji będzie opisanie formalnych kontraktów: ABI, komunikacji driver space, cyklu życia sterownika i dokładnego łańcucha alokacji. Obecna wersja stanowi mapę projektu i bezpieczny punkt wyjścia do tych bardziej szczegółowych specyfikacji.

## Referencje

[1]: https://github.com/CTRL-F-0rg3/TrangorgeOS/tree/unstable "TrangorgeOS — gałąź unstable na GitHubie"

[2]: https://trangorgeos.website/ "Oficjalna strona TrangorgeOS"

[3]: https://github.com/CTRL-F-0rg3/TrangorgeOS/blob/unstable/kernel/src/driverspaceinit/init/init.rs "TrangorgeOS — przygotowanie driver space"

[4]: https://github.com/CTRL-F-0rg3/TrangorgeOS/blob/unstable/kernel/src/driverspaceinit/init/service.rs "TrangorgeOS — usługi driver space"

[5]: https://github.com/CTRL-F-0rg3/TrangorgeOS/blob/unstable/kernel/src/mm/alloc/heap/heap.c "TrangorgeOS — routing sterty buddy/slab"

[6]: https://github.com/CTRL-F-0rg3/TrangorgeOS/blob/unstable/kernel/src/mm/alloc/api/alloc.c "TrangorgeOS — publiczne API alokatora"

[7]: https://github.com/CTRL-F-0rg3/TrangorgeOS/blob/unstable/kernel/src/mm/core/mm.c "TrangorgeOS — inicjalizacja podsystemu pamięci"

[8]: https://github.com/CTRL-F-0rg3/TrangorgeOS "TrangorgeOS — repozytorium projektu"
