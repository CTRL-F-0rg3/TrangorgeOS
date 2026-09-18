#ifndef DP_OPERATION_H
#define DP_OPERATION_H

#include "dp.h"

void dp_op_set_fb(uint64_t phys, uint32_t w, uint32_t h, uint32_t stride);
void dp_op_state(uint64_t *phys, uint32_t *w, uint32_t *h, uint32_t *stride);
void dp_op_fill(uint32_t color, uint32_t x, uint32_t y, uint32_t w, uint32_t h);

#endif
