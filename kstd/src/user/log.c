#include "kstd.h"
#include "dsabi.h"

static volatile char *scratch = (volatile char *)DS_SCRATCH_VA;

void tr_log(const char *s)
{
    uint32_t n = 0;

    while (s[n] && n < DS_SCRATCH_SIZE - 1) {
        scratch[n] = s[n];
        n++;
    }

    ds_call(SVC_SYS, OP_LOG, n, 0, 0);
}