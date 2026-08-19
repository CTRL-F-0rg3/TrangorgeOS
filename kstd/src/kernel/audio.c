#include "kstd_audio.h"

extern int32_t k_audio_play(uint64_t phys, uint32_t len);
extern int32_t k_audio_stop(void);
extern int32_t k_audio_jack(void);
extern int32_t k_audio_amp(int32_t on);
extern uint64_t kvirt_to_phys(void *ptr);

tr_status_t tr_audio_play(const void *data, uint32_t len)
{
    uint64_t phys = kvirt_to_phys((void *)data);

    if (phys == (uint64_t)-1) {
        return TR_ERR_INVALID;
    }

    return k_audio_play(phys, len) == 0 ? TR_OK : TR_ERR_IO;
}

tr_status_t tr_audio_stop(void)
{
    k_audio_stop();
    return TR_OK;
}

tr_status_t tr_audio_jack(bool *present)
{
    if (!present) {
        return TR_ERR_INVALID;
    }

    *present = k_audio_jack() == 1;
    return TR_OK;
}

tr_status_t tr_audio_amp(bool on)
{
    k_audio_amp(on ? 1 : 0);
    return TR_OK;
}