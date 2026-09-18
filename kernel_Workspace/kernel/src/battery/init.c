#include "battery.h"
#include "operation.h"

static bool is_ready = false;

bool battery_init(void)
{
    if (is_ready) {
        return true;
    }

    is_ready = true;

    return true;
}
