#include "camera.h"

static bool is_ready = false;

bool camera_init(void)
{
    if (is_ready) {
        return true;
    }

    is_ready = true;

    return true;
}
