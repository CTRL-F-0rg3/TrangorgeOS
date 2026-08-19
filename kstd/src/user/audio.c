#include "kstd_audio.h"
#include "kstd_mem.h"
#include "dsabi.h"

extern uint64_t ds_call(uint32_t cls, uint32_t op,
                        uint64_t a0, uint64_t a1, uint64_t a2);
extern void ds_poll(void);
extern int ds_take(uint64_t id, ds_msg_t *out);

static void *bounce = TR_NULL;
static uint64_t bounce_phys = 0;

tr_status_t tr_audio_play(const void *data, uint32_t len)
{
    if (bounce == TR_NULL) {
        bounce = tr_alloc(4096 * 4);

        if (bounce == TR_NULL) {
            return TR_ERR_NOMEM;
        }

        uint64_t id = ds_call(SVC_SYS, OP_PAGEPHYS, (uint64_t)bounce, 0, 0);
        ds_poll();

        ds_msg_t r;

        if (!ds_take(id, &r) || r.status != 0) {
            return TR_ERR_IO;
        }

        bounce_phys = r.arg0;
    }

    if (len > 4096 * 4) {
        len = 4096 * 4;
    }

    uint8_t *dst = (uint8_t *)bounce;
    const uint8_t *src = (const uint8_t *)data;

    for (uint32_t i = 0; i < len; i++) {
        dst[i] = src[i];
    }

    uint64_t id = ds_call(SVC_AUDIO, AUD_PLAY, bounce_phys, len, 0);
    ds_poll();

    ds_msg_t r;

    if (ds_take(id, &r) && r.status == 0) {
        return TR_OK;
    }

    return TR_ERR_DENIED;
}

tr_status_t tr_audio_stop(void)
{
    ds_call(SVC_AUDIO, AUD_STOP, 0, 0, 0);
    return TR_OK;
}

tr_status_t tr_audio_jack(bool *present)
{
    if (!present) {
        return TR_ERR_INVALID;
    }

    uint64_t id = ds_call(SVC_AUDIO, AUD_JACK, 0, 0, 0);
    ds_poll();

    ds_msg_t r;

    if (ds_take(id, &r) && r.status == 0) {
        *present = (r.arg0 & 1) != 0;
        return TR_OK;
    }

    return TR_ERR_IO;
}

tr_status_t tr_audio_amp(bool on)
{
    ds_call(SVC_AUDIO, AUD_AMP, on ? 1 : 0, 0, 0);
    return TR_OK;
}