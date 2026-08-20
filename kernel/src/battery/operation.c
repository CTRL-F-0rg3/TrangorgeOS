#include "operation.h"

static battery_status_t model = {
    .present = 1,
    .state = BATT_STATE_DISCHARGING,
    .percent = 87,
    .voltage_mv = 12000,
    .temp_c = 32,
    .health = 96,
};

static uint32_t low_threshold = 10;
static bool low_pending = false;
static uint32_t tick_counter = 0;

static bool model_present(void)
{
    return model.present != 0;
}

static bool model_status(battery_status_t *out)
{
    if (!model.present) {
        return false;
    }

    *out = model;
    return true;
}

static const battery_backend_t model_backend = {
    .present = model_present,
    .status = model_status,
};

static const battery_backend_t *backend = &model_backend;

void battery_register_backend(const battery_backend_t *b)
{
    if (b != (void *)0 && b->status != (void *)0) {
        backend = b;
    }
}

bool battery_present(void)
{
    return backend->present();
}

bool battery_status(battery_status_t *out)
{
    if (out == (void *)0) {
        return false;
    }

    return backend->status(out);
}

bool battery_status_packed(uint64_t *a0, uint64_t *a1, uint64_t *a2)
{
    battery_status_t s;

    if (!battery_status(&s)) {
        return false;
    }

    *a0 = ((uint64_t)s.present << 32)
        | ((uint64_t)s.state << 24)
        | (s.percent & 0xFF);

    *a1 = ((uint64_t)s.voltage_mv << 16) | (s.temp_c & 0xFFFF);
    *a2 = s.health;

    return true;
}

bool battery_set_threshold(uint32_t low_pct)
{
    if (low_pct > 50) {
        return false;
    }

    low_threshold = low_pct;
    low_pending = false;

    return true;
}

uint32_t battery_threshold(void)
{
    return low_threshold;
}

bool battery_low_pending(void)
{
    bool p = low_pending;
    low_pending = false;
    return p;
}

void battery_sim_tick(void)
{
    if (!model.present || model.state == BATT_STATE_CHARGING) {
        return;
    }

    tick_counter++;

    if (tick_counter % 2000 == 0 && model.percent > 0) {
        model.percent--;

        if (model.percent <= low_threshold && !low_pending) {
            low_pending = true;
        }
    }
}