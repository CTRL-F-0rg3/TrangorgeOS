/*
 * Hostowy test jednostkowy dla core/range.c (range_from_addr_len).
 *
 * Nie wymaga cross-toolchainu ani jądra — kompiluje się i uruchamia
 * zwykłym `gcc`/`clang` na hoście, zgodnie z sekcją 7 planu ulepszeń MM
 * ("testy jednostkowe możliwe do uruchamiania poza jądrem dla ...
 * regionów, alignmentu i obliczeń rozmiaru").
 *
 * Budowanie i uruchomienie:
 *   gcc -std=c11 -Wall -Wextra -I ../../src/mm/core \
 *       test_range.c ../../src/mm/core/range.c -o test_range
 *   ./test_range
 */

#include <stdio.h>
#include <stdint.h>
#include "range.h"

static int failures = 0;

#define CHECK(cond, msg)                                                    \
    do {                                                                    \
        if (!(cond)) {                                                      \
            printf("FAIL: %s (linia %d)\n", (msg), __LINE__);               \
            failures++;                                                    \
        } else {                                                            \
            printf("OK:   %s\n", (msg));                                    \
        }                                                                   \
    } while (0)

#define PAGE 0x1000ULL
#define USER_MIN 0x1000ULL
#define USER_MAX 0x00007FFF00000000ULL

int main(void)
{
    uint64_t start, end;

    /* --- P0.3: len == 0 jest odrzucane. --- */
    CHECK(!range_from_addr_len(0x2000, 0, PAGE, USER_MIN, USER_MAX, true,
                               &start, &end),
         "len == 0 jest odrzucane");

    /* --- P0.3: addr + len przepełnia u64 -> odmowa, nie zawinięcie. --- */
    CHECK(!range_from_addr_len(UINT64_MAX - 10, 100, PAGE, 0, UINT64_MAX,
                               false, &start, &end),
         "addr + len > UINT64_MAX jest odrzucane (bez zawijania)");

    CHECK(!range_from_addr_len(UINT64_MAX, 1, PAGE, 0, UINT64_MAX, false,
                               &start, &end),
         "addr == UINT64_MAX, len == 1 jest odrzucane");

    /* --- Zwykły, poprawny zakres wewnątrz jednej strony. --- */
    CHECK(range_from_addr_len(0x2010, 0x20, PAGE, USER_MIN, USER_MAX, true,
                              &start, &end) &&
         start == 0x2000 && end == 0x3000,
         "zwykly zakres jest wyrownywany do granic strony [0x2000,0x3000)");

    /* --- Wyrównanie końca, które samo mogłoby przepełnić u64. --- */
    {
        uint64_t addr = UINT64_MAX - PAGE + 2; /* koniec tuz przed max */
        uint64_t len = 1;
        /* addr + len nie przepełnia (mieści się w u64), ale wyrównanie
         * end w górę do najbliższej wielokrotności PAGE mogłoby. */
        CHECK(!range_from_addr_len(addr, len, PAGE, 0, UINT64_MAX, false,
                                   &start, &end),
             "wyrownanie konca w gore, ktore samo przepelnia, jest odrzucane");
    }

    /* --- P0.3: adresy niekanoniczne x86_64 odrzucane, gdy wymagane. --- */
    CHECK(!range_is_canonical(0x0001000000000000ULL),
         "adres tuz nad granica kanoniczna (2^47) NIE jest kanoniczny");
    CHECK(range_is_canonical(0x00007FFFFFFFFFFFULL),
         "najwyzszy dodatni adres kanoniczny (2^47 - 1) JEST kanoniczny");
    CHECK(range_is_canonical(0xFFFF800000000000ULL),
         "poczatek gornej polowy kanonicznej (0xFFFF8000...) JEST kanoniczny");

    CHECK(!range_from_addr_len(0x0001000000000000ULL, PAGE, PAGE,
                               0, UINT64_MAX, true, &start, &end),
         "range_from_addr_len odrzuca niekanoniczny adres, gdy require_canonical");

    /* --- P0.3: przejscie przez gorna granice user space (USER_MAX). --- */
    CHECK(!range_from_addr_len(USER_MAX - PAGE, 2 * PAGE, PAGE,
                               USER_MIN, USER_MAX, true, &start, &end),
         "zakres wychodzacy poza USER_MAX jest odrzucany");

    /* --- Zakres dokladnie na granicy USER_MAX jest akceptowany. --- */
    CHECK(range_from_addr_len(USER_MAX - PAGE, PAGE, PAGE,
                              USER_MIN, USER_MAX, true, &start, &end) &&
         end == USER_MAX,
         "zakres konczacy sie dokladnie na USER_MAX jest akceptowany");

    /* --- align musi byc potega dwojki. --- */
    CHECK(!range_from_addr_len(0x1000, 0x100, 3, 0, UINT64_MAX, false,
                               &start, &end),
         "align niebedacy potega dwojki jest odrzucany");

    if (failures == 0) {
        printf("\nWszystkie testy przeszly.\n");
    } else {
        printf("\n%d test(y) NIE powiodly sie.\n", failures);
    }

    return failures == 0 ? 0 : 1;
}
