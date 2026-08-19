#include "link.h"
#include "aux.h"

static uint32_t cur_rate = 0;
static uint32_t cur_lanes = 0;

bool dp_link_train(void)
{
    uint8_t max_rate = 0;
    uint8_t max_lanes = 0;

    dp_aux_read(DPCD_MAX_LINK_RATE, &max_rate, 1);
    dp_aux_read(DPCD_MAX_LANE_COUNT, &max_lanes, 1);

    uint8_t bw = max_rate < 0x14 ? max_rate : 0x14;
    uint8_t lanes = max_lanes & 0x1F;

    if (lanes > 4) {
        lanes = 4;
    }

    dp_aux_write(DPCD_LINK_BW_SET, &bw, 1);
    dp_aux_write(DPCD_LANE_COUNT_SET, &lanes, 1);

    uint8_t tp = 1;
    dp_aux_write(DPCD_TRAINING_SET, &tp, 1);

    uint8_t st[2] = { 0, 0 };

    for (int i = 0; i < 5; i++) {
        dp_aux_read(DPCD_LANE_STATUS, st, 2);

        if ((st[0] & 0x11) == 0x11 && (st[1] & 0x11) == 0x11) {
            break;
        }
    }

    tp = 2;
    dp_aux_write(DPCD_TRAINING_SET, &tp, 1);

    for (int i = 0; i < 5; i++) {
        dp_aux_read(DPCD_LANE_STATUS, st, 2);

        if ((st[0] & 0x66) == 0x66 && (st[1] & 0x66) == 0x66) {
            break;
        }
    }

    uint8_t lock = 0;
    dp_aux_read(DPCD_SYMBOL_LOCK, &lock, 1);

    if (lock != 1) {
        return false;
    }

    tp = 0;
    dp_aux_write(DPCD_TRAINING_SET, &tp, 1);

    switch (bw) {
    case 0x0A: cur_rate = 2700; break;
    case 0x14: cur_rate = 5400; break;
    default:   cur_rate = 1620; break;
    }

    cur_lanes = lanes;

    return true;
}

void dp_link_info(uint32_t *rate_mbps, uint32_t *lanes)
{
    *rate_mbps = cur_rate;
    *lanes = cur_lanes;
}