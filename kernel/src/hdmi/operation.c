#include "operation.h"

extern void *arch_phys_to_virt(uint64_t phys);

static hdmi_caps_t cur;
static hdmi_mode_t cur_mode;
static bool op_ready = false;

static hdmi_transfer_t slots[HDMI_MAX_TRANSFERS];
static uint64_t slot_seq[HDMI_MAX_TRANSFERS];
static bool slot_used[HDMI_MAX_TRANSFERS];
static bool slot_done[HDMI_MAX_TRANSFERS];

static uint64_t next_seq = 1;
static uint32_t pending = 0;

void hdmi_op_state(hdmi_caps_t *out)
{
    *out = cur;
}

void hdmi_op_set_fb(uint64_t phys, uint32_t w, uint32_t h, uint32_t stride)
{
    cur.fb_phys = phys;
    cur.w = w;
    cur.h = h;
    cur.stride = stride;
    cur.ready = 1;
}

void hdmi_op_exec(hdmi_transfer_t *t)
{
    uint32_t stride = t->stride ? t->stride : cur.stride;

    uint32_t *dst = (uint32_t *)arch_phys_to_virt(
        t->dst_phys ? t->dst_phys : cur.fb_phys);

    switch (t->kind) {
    case HDMI_TR_FILL:
        for (uint32_t y = t->y; y < t->y + t->h && y < cur.h; y++) {
            for (uint32_t x = t->x; x < t->x + t->w && x < cur.w; x++) {
                dst[y * stride + x] = t->color;
            }
        }
        break;

    case HDMI_TR_BLIT: {
        uint32_t *src = (uint32_t *)arch_phys_to_virt(t->src_phys);

        for (uint32_t y = 0; y < t->h && t->y + y < cur.h; y++) {
            for (uint32_t x = 0; x < t->w && t->x + x < cur.w; x++) {
                dst[(t->y + y) * stride + t->x + x] = src[y * t->stride + x];
            }
        }
        break;
    }

    case HDMI_TR_FLIP:
        cur.fb_phys = t->dst_phys;
        break;
    }
}

uint64_t hdmi_submit(const hdmi_transfer_t *t)
{
    if (!op_ready || pending >= HDMI_MAX_TRANSFERS) {
        return 0;
    }

    for (int i = 0; i < HDMI_MAX_TRANSFERS; i++) {
        if (!slot_used[i]) {
            slots[i] = *t;
            slot_seq[i] = next_seq++;
            slot_used[i] = true;
            slot_done[i] = false;
            pending++;

            hdmi_op_exec(&slots[i]);
            slot_done[i] = true;

            return slot_seq[i];
        }
    }

    return 0;
}

bool hdmi_poll(uint64_t *out_seq)
{
    for (int i = 0; i < HDMI_MAX_TRANSFERS; i++) {
        if (slot_used[i] && slot_done[i]) {
            *out_seq = slot_seq[i];
            slot_used[i] = false;
            slot_done[i] = false;
            pending--;
            return true;
        }
    }

    return false;
}

uint32_t hdmi_pending(void)
{
    return pending;
}

uint64_t hdmi_submit_fill(uint32_t color, uint32_t x, uint32_t y,
                          uint32_t w, uint32_t h)
{
    hdmi_transfer_t t = {
        .kind = HDMI_TR_FILL,
        .color = color,
        .x = x, .y = y, .w = w, .h = h,
    };

    return hdmi_submit(&t);
}

void hdmi_op_set_mode(const hdmi_mode_t *m)
{
    cur.w = m->w;
    cur.h = m->h;
    cur.stride = m->w;
}

void hdmi_caps_raw(uint32_t *w, uint32_t *h, uint32_t *s, uint64_t *phys)
{
    hdmi_caps_t c;
    hdmi_caps(&c);
    *w = c.w; *h = c.h; *s = c.stride; *phys = c.fb_phys;
}