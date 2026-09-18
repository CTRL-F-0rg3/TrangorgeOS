#ifndef BATTERY_OPERATION_H
#define BATTERY_OPERATION_H

#include "battery.h"

typedef struct battery_backend {
    bool (*present)(void);
    bool (*status)(battery_status_t *out);
} battery_backend_t;

void battery_register_backend(const battery_backend_t *b);

#endif
