#ifndef HDMI_OPERATION_H
#define HDMI_OPERATION_H
#include "mode.h"

void hdmi_op_set_mode(const hdmi_mode_t *m);
#include "hdmi.h"

void hdmi_op_exec(hdmi_transfer_t *t);
void hdmi_op_state(hdmi_caps_t *out);
void hdmi_op_set_fb(uint64_t phys, uint32_t w, uint32_t h, uint32_t stride);

#endif