#ifndef DSLOG_H
#define DSLOG_H
#include "dsabi.h"

static inline void ds_log(const char *s)
{
    volatile char *scratch = (volatile char *)DS_SCRATCH_VA;
    uint64_t n = 0;

    while (s[n] != '\0' && n < DS_SCRATCH_SIZE - 1) {
        scratch[n] = s[n];
        n++;
    }

    ds_call(SVC_SYS, OP_LOG, n, 0, 0);
}

#endif