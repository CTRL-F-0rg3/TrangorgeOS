#ifndef MM_CORE_SIZEUTIL_H
#define MM_CORE_SIZEUTIL_H

#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>

bool size_is_pow2(size_t v);


bool size_round_up_pow2(size_t v, size_t *out);


bool kfree_pages_validate(size_t pages,
                          size_t page_size,
                          size_t usable_bytes,
                          size_t *out_requested_bytes);

bool size_bytes_to_pages_checked(size_t bytes,
                                 size_t page_size,
                                 size_t *out_pages);

bool size_pages_to_bytes_checked(size_t pages,
                                 size_t page_size,
                                 size_t *out_bytes);

#endif
