#include "range.h"

bool range_is_canonical(uint64_t addr)
{

    uint64_t top = addr >> 47;

    return top == 0ULL || top == 0x1FFFFULL;
}

static bool is_pow2(uint64_t v)
{
    return v != 0 && (v & (v - 1)) == 0;
}

bool range_from_addr_len(uint64_t addr,
                         uint64_t len,
                         uint64_t align,
                         uint64_t limit_min,
                         uint64_t limit_max,
                         bool require_canonical,
                         uint64_t *out_start,
                         uint64_t *out_end)
{
    if (out_start == NULL || out_end == NULL) {
        return false;
    }

    if (len == 0) {
        return false;
    }

    if (!is_pow2(align)) {
        return false;
    }

    /* (1) addr + len bez przepełnienia u64. */
    if (addr > UINT64_MAX - len) {
        return false;
    }

    uint64_t raw_end = addr + len;

    /* (2) wyrównanie początku w dół — tylko czyści bity, nie przepełnia. */
    uint64_t start = addr & ~(align - 1);

    /* (3) wyrównanie końca w górę — jawnie sprawdzone pod kątem overflow. */
    uint64_t misalign = raw_end & (align - 1);
    uint64_t end;

    if (misalign == 0) {
        end = raw_end;
    } else {
        uint64_t pad = align - misalign;

        if (raw_end > UINT64_MAX - pad) {
            return false;
        }

        end = raw_end + pad;
    }

    if (end <= start) {
        return false;
    }

    /* (4) kanoniczność (opcjonalna — np. helper bywa też używany dla
     * zakresów fizycznych/wewnętrznych, gdzie pojęcie "kanoniczny" nie
     * ma zastosowania). */
    if (require_canonical) {
        if (!range_is_canonical(start) || !range_is_canonical(end - 1)) {
            return false;
        }
    }

    /* (5) granice przestrzeni narzucone przez wywołującego. */
    if (start < limit_min) {
        return false;
    }

    if (end > limit_max) {
        return false;
    }

    *out_start = start;
    *out_end = end;

    return true;
}