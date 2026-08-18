#ifndef DSAUDIO_H
#define DSAUDIO_H
#include "dsabi.h"

static inline uint64_t ds_audio_play(uint64_t phys, uint32_t len)
{
    return ds_call(SVC_AUDIO, AUD_PLAY, phys, len, 0);
}

static inline void ds_audio_stop(void)
{
    ds_call(SVC_AUDIO, AUD_STOP, 0, 0, 0);
}

static inline uint64_t ds_audio_jack_req(void)
{
    return ds_call(SVC_AUDIO, AUD_JACK, 0, 0, 0);
}

static inline void ds_audio_amp(int on)
{
    ds_call(SVC_AUDIO, AUD_AMP, on ? 1 : 0, 0, 0);
}

#endif