#ifndef DP_AUX_H
#define DP_AUX_H

#include <stdint.h>
#include <stdbool.h>

#define DPCD_REV             0x00000
#define DPCD_MAX_LINK_RATE   0x00001
#define DPCD_MAX_LANE_COUNT  0x00002
#define DPCD_LINK_BW_SET     0x00100
#define DPCD_LANE_COUNT_SET  0x00101
#define DPCD_TRAINING_SET    0x00102
#define DPCD_LANE_ADJUST     0x00200
#define DPCD_LANE_STATUS     0x00202
#define DPCD_SYMBOL_LOCK     0x00204

void dp_aux_sink_init(void);
bool dp_aux_read(uint32_t addr, void *buf, uint32_t len);
bool dp_aux_write(uint32_t addr, const void *buf, uint32_t len);
bool dp_aux_read_edid(void *buf, uint32_t len);

#endif