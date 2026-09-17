#ifndef DP_LINK_H
#define DP_LINK_H

#include <stdint.h>
#include <stdbool.h>

bool dp_link_train(void);
void dp_link_info(uint32_t *rate_mbps, uint32_t *lanes);

#endif
