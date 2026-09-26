#ifndef MATH_CLOC_H
#define MATH_CLOC_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct {
    uint64_t secs;
    uint32_t nanos;
} math_cloc_time_t;

#define MATH_CLOC_SUCCESS          0
#define MATH_CLOC_ERR_NULL_PTR    -1
#define MATH_CLOC_ERR_OVERFLOW    -2
#define MATH_CLOC_ERR_TIME_REWIND -3

int32_t math_cloc_add(math_cloc_time_t a, math_cloc_time_t b, math_cloc_time_t* out);
int32_t math_cloc_validate_monotonic(math_cloc_time_t current, math_cloc_time_t previous);

#ifdef __cplusplus
}
#endif

#endif /* MATH_CLOC_H */