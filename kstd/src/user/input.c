#include "kstd_input.h"
#include "dsabi.h"

extern uint64_t ds_call(uint32_t cls, uint32_t op,
                        uint64_t a0, uint64_t a1, uint64_t a2);
extern void ds_poll(void);
extern int ds_take(uint64_t id, ds_msg_t *out);

int32_t tr_input_key(void)
{
    uint64_t id = ds_call(SVC_INPUT, IN_KEY_POLL, 0, 0, 0);
    ds_poll();

    ds_msg_t r;

    if (ds_take(id, &r) && r.status == 0 && r.arg0 != 0) {
        return (int32_t)r.arg0;
    }

    return -1;
}