#ifndef BATTERY_H
#define BATTERY_H

#include <stdint.h>
#include <stdbool.h>
#include <stddef.h>

#define BATT_STATE_DISCHARGING 0
#define BATT_STATE_CHARGING    1
#define BATT_STATE_FULL        2

typedef struct battery_status {
    uint32_t present;
    uint32_t state;
    uint32_t percent;
    uint32_t voltage_mv;
    uint32_t temp_c;
    uint32_t health;
} battery_status_t;

bool battery_init(void);
bool battery_present(void);
bool battery_status(battery_status_t *out);
bool battery_status_packed(uint64_t *a0, uint64_t *a1, uint64_t *a2);
bool battery_set_threshold(uint32_t low_pct);
uint32_t battery_threshold(void);
bool battery_low_pending(void);
void battery_sim_tick(void);

#endif