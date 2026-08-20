#ifndef MM_CORE_SIZEUTIL_H
#define MM_CORE_SIZEUTIL_H

#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>

/*
 * Wspólne, hostowo-testowalne helpery do walidacji rozmiarów/wyrównań —
 * użyte przy naprawie P1.1 (niejednoznaczny kontrakt `kmalloc_aligned`).
 */

/* Zwraca true, gdy `v` jest potęgą dwójki (0 NIE jest potęgą dwójki). */
bool size_is_pow2(size_t v);

/*
 * Zaokrągla `v` w górę do najbliższej potęgi dwójki i zapisuje wynik w
 * *out. Zwraca false (bez modyfikowania *out), jeśli wynik nie mieści się
 * w size_t — NIGDY nie zawija się cicho do małej/zerowej wartości, w
 * przeciwieństwie do pętli `p <<= 1` bez sprawdzenia przepełnienia.
 * v == 0 daje *out == 1.
 */
bool size_round_up_pow2(size_t v, size_t *out);

/*
 * Waliduje parametr `pages` przekazany do kfree_pages() (P1.2) względem
 * rzeczywistej pojemności bloku (`usable_bytes`, zwykle z
 * heap_usable_size()) leżącego pod wskaźnikiem, który ma zostać
 * zwolniony. Czysta funkcja — bez zależności od jądra — więc testowalna
 * na hoście.
 *
 * Zwraca false (odmowa zwolnienia), gdy:
 *   - pages == 0,
 *   - pages * page_size przepełniłby size_t,
 *   - usable_bytes != 0 i żądane bajty przekraczają pojemność bloku
 *     (usable_bytes == 0 oznacza "nierozpoznany wskaźnik" — walidacja
 *     jest wtedy pomijana, bo i tak nie ma z czym porównać; docelowy
 *     `heap_free()` bezpiecznie zignoruje nierozpoznany adres).
 *
 * Przy sukcesie zapisuje pages * page_size w *out_requested_bytes.
 */
bool kfree_pages_validate(size_t pages,
                          size_t page_size,
                          size_t usable_bytes,
                          size_t *out_requested_bytes);

/*
 * P1.4: bezpieczna konwersja bytes -> pages (zaokrąglenie w górę), bez
 * cichego zawijania przy `bytes` bliskich SIZE_MAX. Zastępuje wzorzec
 * `(bytes + page_size - 1) / page_size` używany bez ochrony w
 * `contig_bytes_to_frames()`/`dma_bytes_to_frames()`.
 */
bool size_bytes_to_pages_checked(size_t bytes,
                                 size_t page_size,
                                 size_t *out_pages);

/*
 * P1.4: bezpieczna konwersja pages -> bytes (mnożenie z jawną kontrolą
 * przepełnienia), używana np. do policzenia długości mapowania z liczby
 * ramek.
 */
bool size_pages_to_bytes_checked(size_t pages,
                                 size_t page_size,
                                 size_t *out_bytes);

#endif
