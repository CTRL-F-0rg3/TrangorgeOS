# Plan ulepszenia podsystemu MM w TrangorgeOS

**Repozytorium:** [CTRL-F-0rg3/TrangorgeOS](https://github.com/CTRL-F-0rg3/TrangorgeOS)  
**Analizowana gałąź:** `stabilizing`  
**Analizowany commit:** `7a0d836f2180874aac3e8c92493c791aec89a8b3`  
**Autor analizy:** ctrl 
**Data analizy:** 20 sierpnia 2026 r.
 
 
 masz kropke gdzie ostatnio dodałem


## 1. Zakres i podsumowanie

Przeanalizowany został cały katalog `kernel/src/mm`, w tym: inicjalizacja pamięci architektury, bitmapowy PMM, allocator ramek, mapowanie wirtualne, paging, TLB, sterta buddy/slab, API `kmalloc`, debug allocator, DMA, pamięć ciągła, cache, ochrona oraz przestrzenie adresowe procesów.

Architektura jest sensownie rozdzielona na warstwy. Obecny przepływ startowy (`arch_memory_init` → paging → PMM → VMM → heap → cache → paging subsystem → isolation → address spaces) daje dobrą bazę do dalszego rozwoju. Największym problemem nie jest brak komponentów, lecz to, że część API jest jeszcze prototypowa: ścieżka slab jest wyłączona, blokady nie zapewniają bezpieczeństwa SMP, operacje zakresowe nie wszędzie sprawdzają overflow, a obsługa VMA/mmap nie ma jeszcze pełnej semantyki systemowej.

> **Najważniejsza rekomendacja:** przed dodawaniem nowych funkcji należy ustabilizować kontrakty allocatorów, walidację zakresów i synchronizację. Bez tego rozwój procesów, sterowników i DMA będzie zwiększał ryzyko cichych uszkodzeń pamięci.

## 2. Ocena stanu obecnego

| Obszar | Stan | Ocena |
|---|---|---|
| Rozdzielenie PMM/VMM/heap/process | Istnieje i jest czytelne | Dobra baza architektoniczna |
| Bitmapa ramek | Obsługuje pojedyncze i ciągłe zakresy | Wymaga testów granicznych i lepszej wydajności |
| Buddy allocator | Działa jako główna ścieżka heap | Brak synchronizacji SMP i dokładnego rozmiaru żądania |
| Slab allocator | Zaimplementowany, ale `HEAP_USE_SLAB` wynosi `0` | Niewykorzystana optymalizacja; ma ryzyko double-free |
| API alokacji | `kmalloc`, `kzalloc`, `kcalloc`, `krealloc`, strony | Kontrakty `aligned/pages` są niepełne |
| Paging/VMM | Są mapowanie, translate, protect i address spaces | Brakuje pełnej transakcyjności i walidacji overflow |
| DMA/contiguous | Istnieją osobne moduły | Wymagają bezpiecznego liczenia rozmiarów i modelu cache |
| Izolacja | SMEP/SMAP/NX i audyt PML4 | Włączenie mechanizmów nie jest raportowane jako błąd krytyczny |
| Testy | Jest `kernel/src/testing.rs` i kilka self-testów | Brak automatycznej, szerokiej macierzy testów MM |
| Build/CI | W środowisku analizy `cargo` nie było dostępne | Nie udało się potwierdzić kompilacji; potrzebny CI z cross-toolchainem |

## 3. Problemy wymagające naprawy

### P0 — bezpieczeństwo i poprawność krytyczna

#### P0.1. Blokady wyłączające przerwania nie są blokadami SMP

PMM, slab i przestrzenie adresowe używają wzorca `pushfq; cli` oraz lokalnego licznika zagnieżdżenia. Wyłączenie przerwań chroni stan tylko przed przerwaniem na bieżącym CPU; nie chroni przed drugim rdzeniem. Przy konfiguracji wielordzeniowej (`-smp 4` w konfiguracji uruchomieniowej) dwa CPU mogą jednocześnie zmienić bitmapę, listy wolnych bloków lub listę VMA.

**Pliki:** `kernel/src/mm/alloc/physical/pmm.c`, `kernel/src/mm/alloc/heap/slab.c`, `kernel/src/mm/process/address_space.c`.

**Zadanie:** zastąpić blokady per-CPU rzeczywistymi spinlockami ticket/MCS albo istniejącym prymitywem SMP z projektu. Zachować wariant `irqsave`, ale używać go dopiero razem ze spinlockiem. Udokumentować kolejność blokad, aby uniknąć deadlocków PMM → mapping → heap.

**Kryterium akceptacji:** test wielordzeniowy wykonuje równolegle co najmniej 100 000 alokacji i zwolnień z wielu CPU; bitmapa, liczniki i listy wolnych bloków pozostają spójne.

#### P0.2. Double-free w slabie może uszkodzić listę wolnych obiektów

`slab_free()` sprawdza zakres i wyrównanie wskaźnika, ale nie przechowuje informacji, czy konkretny obiekt jest już wolny. Dwukrotne zwolnienie tego samego adresu ponownie wpisuje go do free-listy i może zwiększyć `free_count` ponad `objects_per_slab`. Należy dodać bitmapę zajętości per slab albo bezpieczny stan obiektu w trybie debug.

**Plik:** `kernel/src/mm/alloc/heap/slab.c`, szczególnie ścieżka `slab_free()`.

**Kryterium akceptacji:** drugie `slab_free(ptr)` nie modyfikuje allocatorów i generuje błąd diagnostyczny; test obejmuje także zwolnienie wskaźnika z wnętrza obiektu.

#### P0.3. Brak walidacji overflow w API przestrzeni adresowych

W wielu miejscach granice są liczone jako `addr + len`, a następnie wyrównywane, np. w `aspace_map_at`, `aspace_unmap`, `aspace_protect` i `aspace_reserve_at`. Overflow może zmienić zakres na mały lub niepoprawny i ominąć oczekiwaną walidację użytkownika.

**Plik:** `kernel/src/mm/process/address_space.c`, okolice obliczeń `a`/`b`.

**Zadanie:** wprowadzić wspólną funkcję `range_from_addr_len(addr, len, alignment, out_start, out_end)`, która:

1. odrzuca `len == 0`;
2. sprawdza `addr + len` bez overflow;
3. bezpiecznie wyrównuje początek w dół i koniec w górę;
4. sprawdza kanoniczność oraz pełne granice user space;
5. zwraca zakres półotwarty `[start, end)`.

**Kryterium akceptacji:** testy dla `UINT64_MAX`, adresów niekanonicznych, przejścia przez granicę user/kernel i długości powodujących overflow kończą się odmową operacji.

#### P0.4. Nieatomowy rollback w `aspace_brk()`

`aspace_brk()` najpierw tworzy lub rozszerza VMA, a dopiero potem wywołuje `map_anon_pages()`. Jeżeli alokacja ramek albo mapowanie zakończy się częściowym błędem, funkcja zwraca stare `brk`, ale powiększona VMA pozostaje. Stan metadanych przestaje wtedy odpowiadać tablicom stron.

**Plik:** `kernel/src/mm/process/address_space.c`, ścieżka powiększania sterty.

**Zadanie:** zastosować transakcję: najpierw przygotować zakres i zmapować strony, a dopiero po pełnym sukcesie zatwierdzić zmianę VMA i `pa->brk`; w razie błędu zwolnić wszystkie częściowo zmapowane strony i nie zmieniać metadanych.

**Kryterium akceptacji:** wymuszony brak ramek w połowie operacji pozostawia identyczne VMA, `brk` i liczbę zajętych ramek jak przed wywołaniem.

### P1 — błędy kontraktów allocatorów

#### P1.1. `kmalloc_aligned()` nie gwarantuje dowolnego wyrównania

`buddy_alloc_aligned()` zaokrągla potrzebny rozmiar do potęgi dwójki i zakłada, że baza buddy jest odpowiednio wyrównana. To działa dla wyrównań będących potęgą dwójki, ale nie dla dowolnego wyrównania przekazanego przez użytkownika. Brakuje też odrzucenia niepoprawnego `align` i ochrony przed overflow podczas `p <<= 1`.

**Pliki:** `kernel/src/mm/alloc/api/alloc.c`, `kernel/src/mm/alloc/heap/buddy.c`.

**Zadanie:** jasno zdefiniować API: albo przyjmować wyłącznie potęgi dwójki i to walidować, albo dodać over-allocation z nagłówkiem zawierającym oryginalny wskaźnik i rozmiar. Dla każdego wariantu dodać `kmalloc_usable_size` i testy wyrównań 1, 8, 4096, 64 KiB oraz wartości niebędących potęgami dwójki.

#### P1.2. `kfree_pages()` ignoruje parametr `pages`

`kfree_pages(void *ptr, size_t pages)` ignoruje `pages` i deleguje do `heap_free()`. To tworzy mylący kontrakt: funkcja wygląda jak para dla alokacji stron, ale faktycznie zwalnia blok buddy na podstawie metadanych. Należy albo usunąć parametr, albo wdrożyć osobny allocator stron z dokładnym śledzeniem długości i ochroną przed niezgodną wartością.

**Plik:** `kernel/src/mm/alloc/api/alloc.c`.

#### P1.3. `krealloc()` nie przechowuje rzeczywistego rozmiaru żądania

Poza debug mode `heap_usable_size()` zwraca rozmiar zaokrąglonego bloku buddy, a nie liczbę bajtów żądanych przez użytkownika. `krealloc()` kopiuje cały rozmiar usable block. Jest to zwykle bezpieczne fizycznie, ale może kopiować nieinicjalizowane dane, a późniejsze allocatory nie mają informacji o rozmiarze logicznym. Warto dodać nagłówek alokacji albo zunifikować metadane debug/release.

#### P1.4. Overflow w alokacji contiguous i DMA

`contig_bytes_to_frames()` i `dma_bytes_to_frames()` wykonują `bytes + ARCH_PAGE_SIZE - 1` bez jawnej ochrony overflow. Późniejsze `frames * ARCH_PAGE_SIZE` również powinno być sprawdzane przed konwersją do długości mapowania.

**Pliki:** `kernel/src/mm/alloc/special/contiguous.c`, `kernel/src/mm/alloc/special/dma.c`.

**Kryterium akceptacji:** największe wartości `size_t`, `UINT64_MAX`, długości niebędące wielokrotnością strony i wyrównania większe od pamięci fizycznej są odrzucane bez zmiany stanu PMM.

### P1 — paging, TLB i mapowania

Należy przeprowadzić audyt wszystkich ścieżek `map`, `unmap` i `protect` pod kątem transakcyjności. Jeżeli mapowanie wielu stron nie powiedzie się po kilku stronach, funkcja powinna odmapować wykonany prefiks i zwolnić jego ramki. Po zmianie uprawnień lub usunięciu mapowania trzeba jawnie zagwarantować poprawne unieważnienie TLB lokalnie oraz na innych CPU, gdy dana przestrzeń adresowa jest aktywna na wielu rdzeniach.

**Zadania:**

| Zadanie | Oczekiwany rezultat |
|---|---|
| Ujednolicić `map_range`/`unmap_range` | Jedna polityka wyrównania, overflow i rollbacku |
| Dodać statusy błędów | Rozróżnienie `invalid`, `already mapped`, `no memory`, `permission` |
| Dokończyć batch TLB | Jedno `invlpg` na adres lub pełny flush przy przepełnieniu batcha |
| Dodać shootdown SMP | IPI lub bezpieczny mechanizm synchronizacji aktywnych CR3 |
| Walidować PTE flags | Zakaz USER w mapowaniach kernela i spójne NX/WRITE/DEVICE |
| Testować aliasy | Mapowanie tej samej ramki pod różne VA z kontrolą uprawnień |

### P1 — semantyka `mmap`/VMA

Obecne `mmap()` rozpoznaje głównie `MAP_FIXED` i kieruje pozostałe przypadki do anonimowego mapowania. `MAP_SHARED`, `MAP_PRIVATE`, `MAP_ANONYMOUS` i pozostałe flagi nie są jeszcze pełną semantyką systemową. `mprotect()` wyszukuje VMA przed wejściem w blokadę, co może być wyścigiem na SMP. Dodatkowo ochrona jest ustawiana dla całego VMA, bez rozbijania VMA na fragmenty, gdy zakres dotyczy tylko części.

**Zadania:**

1. Walidować dozwolone kombinacje flag i odrzucać nieobsługiwane zamiast je ignorować.
2. Rozdzielić VMA na prefix/zakres/suffix przy częściowym `mprotect`.
3. Wykonywać wyszukanie VMA pod tą samą blokadą co modyfikację.
4. Ustalić, czy mapowanie anonimowe ma być eager czy demand-paged.
5. Dodać guard pages dla stosu oraz limit wzrostu stosu.
6. Wprowadzić copy-on-write dla `MAP_PRIVATE`, gdy pojawi się fork/procesy potomne.

## 4. Ulepszenia architektury i wydajności


# .


### 4.1. PMM strefowy zamiast jednej bitmapy dla całego maksimum fizycznego

Obecna bitmapa indeksuje PFN-y aż do najwyższego użytecznego adresu, także przez dziury w fizycznej przestrzeni adresowej. Jest to proste, ale przy dużych i rozproszonych mapach pamięci zużywa niepotrzebne metadane. Kolejny etap powinien wprowadzić strefy `DMA32`, `NORMAL` i opcjonalnie `HIGHMEM`, z osobnymi statystykami oraz polityką preferencji.

### 4.2. Per-CPU cache dla małych alokacji

Po ustabilizowaniu slaba warto dodać per-CPU magazine/cache dla najczęstszych klas 16–256 B. Ograniczy to globalną konkurencję i liczbę wejść do PMM. Uzupełnieniem powinny być batchowe pobieranie i oddawanie obiektów.

### 4.3. Lepszy allocator stron dla dużych bloków

Buddy jest dobrym źródłem dużych bloków, ale obecna sterta mapuje i odmapowuje strony przy każdej operacji. Warto dodać cache pustych stron, opcjonalne lazy unmap oraz osobny `vmap` dla dużych, rzadko używanych zakresów. Każda optymalizacja musi jednak zachować jasny ownership ramki.

### 4.4. Cache i DMA

`dma_alloc_coherent()` potrzebuje jawnego modelu spójności zależnego od architektury. Należy rozróżnić pamięć coherent od streaming DMA oraz zdefiniować operacje `dma_sync_for_device()` i `dma_sync_for_cpu()`. `clflush` powinien być używany tylko wtedy, gdy capabilities CPU i typ mapowania tego wymagają; zakresy muszą być sprawdzane pod kątem overflow.

### 4.5. Bezpieczne flagi ochrony

Należy ustalić jedną reprezentację ochrony. Obecnie występują osobne zestawy `PROT_*` i `virt::*` flags. Warto wprowadzić typ/enum warstwy C oraz cienkie, sprawdzane konwersje w Rust, aby nie dało się pomylić flag użytkownika z flagami PTE.

## 5. Diagnostyka i obserwowalność

Debug allocator ma dobre podstawy: magic, poison, canary, leak table i statystyki. Ograniczeniem jest stałe `LEAK_MAX 2048`, brak synchronizacji oraz brak identyfikacji CPU/obiektu/stack trace. W trybie debug należy:

| Ulepszenie | Cel |
|---|---|
| Rozszerzyć leak table lub użyć hash table | Brak cichego nieśledzenia alokacji po przekroczeniu limitu |
| Dodać `alloc_id` | Jednoznaczne śledzenie życia obiektu |
| Dodać guard pages dla dużych alokacji | Wykrywanie overrun/underrun poza canary |
| Raportować ownera cache i CPU | Debugowanie SMP |
| Dodać histogram rozmiarów | Strojenie klas slab/buddy |
| Wykrywać niezgodną parę allocator/free | Natychmiastowe raportowanie błędów ownership |
| Eksportować snapshot MM | Testy regresyjne: free/used, VMA, mapowania, TLB |

## 6. Plan implementacji etapami

| Etap | Zakres | Priorytet | Definicja ukończenia |
|---|---|---:|---|
| A | Wspólne helpery overflow/range/alignment | P0 | Wszystkie publiczne API zakresowe używają helperów |
| B | Spinlocki SMP + kolejność blokad | P0 | Test równoległy nie wykazuje race/double allocation |
| C | Transakcyjny map/unmap/`aspace_brk` | P0 | Błąd w połowie operacji nie zmienia stanu |
| D | Bezpieczny slab z bitmapą obiektów | P0 | Double-free i invalid-free są wykrywane |
| E | Kontrakty aligned/pages/realloc | P1 | API ma dokumentowane i testowane gwarancje |
| F | Overflow DMA/contiguous + strefy PMM | P1 | Testy graniczne przechodzą, DMA32 działa deterministycznie |
| G | Pełniejsze VMA/mmap/mprotect | P1 | Częściowe zakresy są poprawnie dzielone |
| H | TLB shootdown SMP i batchowanie | P1 | Zmiany mapowania są widoczne na wszystkich CPU |
| I | Per-CPU cache i optymalizacja buddy/slab | P2 | Benchmark pokazuje mniejszą kontencję bez regresji |
| J | Demand paging/COW/huge pages | P2 | Funkcje mają testy integracyjne z procesami |

## 7. Minimalny zestaw testów regresyjnych

Należy dodać testy jednostkowe możliwe do uruchamiania poza jądrem dla bitmapy, regionów, alignmentu i obliczeń rozmiaru. Testy integracyjne powinny działać w QEMU z co najmniej czterema vCPU.

| Grupa | Przypadki |
|---|---|
| Bitmapa | pusta, pełna, jeden bit, granice 63/64/65, zakres przez granicę słowa, double-free |
| PMM | pojedyncze ramki, contiguous, align 0/1/nie-potęga, max address, brak pamięci |
| Buddy | każdy order, koalescencja, fragmentacja, invalid pointer, double-free, overflow align |
| Slab | każda klasa, pełny/pusty slab, double-free, invalid pointer, równoległość |
| API heap | `calloc` overflow, `realloc(NULL)`, `realloc(ptr,0)`, aligned, pages |
| Mapowanie | częściowy błąd, rollback, overlap, unmap fragmentu, protect fragmentu |
| VMA | zakresy graniczne, `brk` rollback, guard page, sąsiednie VMA |
| DMA | 32-bit zone, długość overflow, unmap rollback, sync CPU/device |
| Ochrona | NX, SMEP, SMAP, USER/KERNEL PTE, zakaz W^X |

## 8. Dokumentacja i CI

Dokumentacja architektury powinna używać gałęzi `stabilizing`, a nie nieaktualnych linków do `unstable`. Warto dodać do każdego modułu krótką sekcję ownership: kto alokuje ramkę, kto ją mapuje, kto ją zwalnia i kiedy wolno wykonać flush TLB.

Repozytorium powinno mieć CI, który buduje kernel w tym samym cross-toolchainie co Docker/QEMU, uruchamia testy hostowe bitmapy/regionów oraz wykonuje testy bootowalne w QEMU. Sama kontrola `cargo check` nie wystarczy dla kodu `no_std`, inline assembly i linkowania obrazu systemowego.

## 9. Kolejność rekomendowana dla autora

Najpierw należy wdrożyć helpery zakresów i overflow, następnie spinlocki SMP. Dopiero potem warto naprawić transakcje mapowania i `aspace_brk`, ponieważ poprawna synchronizacja jest warunkiem wiarygodnych testów. Trzecim krokiem powinien być bezpieczny slab i ujednolicenie kontraktów alokacji. Po tej bazie można rozwijać pełne VMA/mmap, DMA, demand paging, COW i optymalizacje per-CPU.

> **Nie rekomenduję jeszcze włączania `HEAP_USE_SLAB` w produkcyjnej ścieżce.** Najpierw trzeba dodać wykrywanie double-free, testy SMP oraz potwierdzić poprawność mapowania i zwalniania pustych slabów.

## 10. Odwołania do kodu

Najważniejsze ustalenia wynikają bezpośrednio z analizowanej gałęzi:

- [API alokacji](https://github.com/CTRL-F-0rg3/TrangorgeOS/blob/stabilizing/kernel/src/mm/alloc/api/alloc.c) — `krealloc`, `kalloc_pages`, `kfree_pages`, `kmalloc_aligned`.
- [Buddy allocator](https://github.com/CTRL-F-0rg3/TrangorgeOS/blob/stabilizing/kernel/src/mm/alloc/heap/buddy.c) — orders, listy wolnych bloków, mapowanie i koalescencja.
- [Slab allocator](https://github.com/CTRL-F-0rg3/TrangorgeOS/blob/stabilizing/kernel/src/mm/alloc/heap/slab.c) — free-list obiektów i zwalnianie pustych slabów.
- [PMM](https://github.com/CTRL-F-0rg3/TrangorgeOS/blob/stabilizing/kernel/src/mm/alloc/physical/pmm.c) oraz [frame allocator](https://github.com/CTRL-F-0rg3/TrangorgeOS/blob/stabilizing/kernel/src/mm/alloc/physical/frame.c) — bitmapa ramek i synchronizacja.
- [Przestrzenie adresowe](https://github.com/CTRL-F-0rg3/TrangorgeOS/blob/stabilizing/kernel/src/mm/process/address_space.c) — VMA, `mmap`, `unmap`, `protect`, `brk`.
- [DMA](https://github.com/CTRL-F-0rg3/TrangorgeOS/blob/stabilizing/kernel/src/mm/alloc/special/dma.c) i [pamięć ciągła](https://github.com/CTRL-F-0rg3/TrangorgeOS/blob/stabilizing/kernel/src/mm/alloc/special/contiguous.c) — specjalne ścieżki alokacji.
- [Dokumentacja architektury](https://github.com/CTRL-F-0rg3/TrangorgeOS/blob/stabilizing/TrangorgeOS%20%E2%80%94%20dokumentacja%20architektury.markdown) — opis warstw i kolejności inicjalizacji.

## References

[1]: https://github.com/CTRL-F-0rg3/TrangorgeOS/tree/stabilizing "TrangorgeOS — gałąź stabilizing"
[2]: https://github.com/CTRL-F-0rg3/TrangorgeOS/commit/7a0d836f2180874aac3e8c92493c791aec89a8b3 "TrangorgeOS — analizowany commit"
[3]: https://github.com/CTRL-F-0rg3/TrangorgeOS/blob/stabilizing/kernel/src/mm/alloc/heap/buddy.c "Buddy allocator"
[4]: https://github.com/CTRL-F-0rg3/TrangorgeOS/blob/stabilizing/kernel/src/mm/alloc/heap/slab.c "Slab allocator"
[5]: https://github.com/CTRL-F-0rg3/TrangorgeOS/blob/stabilizing/kernel/src/mm/process/address_space.c "Address spaces"
[6]: https://github.com/CTRL-F-0rg3/TrangorgeOS/blob/stabilizing/kernel/src/mm/alloc/physical/pmm.c "Physical memory manager"
[7]: https://github.com/CTRL-F-0rg3/TrangorgeOS/blob/stabilizing/kernel/src/mm/alloc/special/dma.c "DMA allocator"
[8]: https://github.com/CTRL-F-0rg3/TrangorgeOS/blob/stabilizing/TrangorgeOS%20%E2%80%94%20dokumentacja%20architektury.markdown "Dokumentacja architektury TrangorgeOS"
