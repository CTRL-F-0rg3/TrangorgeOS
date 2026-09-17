#include "operation.h"

#define DIRECT_BASE 0xFFFF888000000000ULL

static void *phys_to_virt(uint64_t phys)
{
    return (void *)(DIRECT_BASE + phys);
}

static hdmi_caps_t cur;
static hdmi_mode_t cur_mode;
static bool op_ready;
static hdmi_transfer_t slots[HDMI_MAX_TRANSFERS];
static uint64_t slot_seq[HDMI_MAX_TRANSFERS];
static bool slot_used[HDMI_MAX_TRANSFERS];
static bool slot_done[HDMI_MAX_TRANSFERS];
static uint64_t next_seq = 1;
static uint32_t pending;

void hdmi_op_state(hdmi_caps_t *out)
{
    if (out != NULL) {
        *out = cur;
    }
}

bool hdmi_op_ready(void)
{
    return op_ready;
}

void hdmi_op_set_fb(uint64_t phys, uint32_t w, uint32_t h, uint32_t stride)
{
    cur.fb_phys = phys;
    cur.w = w;
    cur.h = h;
    cur.stride = stride;
    cur.ready = (phys != 0 && w != 0 && h != 0 && stride >= w) ? 1 : 0;
    op_ready = cur.ready != 0;
}

void hdmi_op_exec(hdmi_transfer_t *t)
{
    if (t == NULL || !op_ready) {
        return;
    }

    uint32_t stride = t->stride != 0 ? t->stride : cur.stride;
    uint64_t dst_phys = t->dst_phys != 0 ? t->dst_phys : cur.fb_phys;
    if (stride < cur.w || dst_phys == 0) {
        return;
    }

    uint32_t *dst = (uint32_t *)phys_to_virt(dst_phys);
    if (dst == NULL) {
        return;
    }

    if (t->kind == HDMI_TR_FILL) {
        if (t->x >= cur.w || t->y >= cur.h) {
            return;
        }
        uint32_t width = t->w < cur.w - t->x ? t->w : cur.w - t->x;
        uint32_t height = t->h < cur.h - t->y ? t->h : cur.h - t->y;
        for (uint32_t y = 0; y < height; y++) {
            for (uint32_t x = 0; x < width; x++) {
                dst[(t->y + y) * stride + t->x + x] = t->color;
            }
        }
        return;
    }

    if (t->kind == HDMI_TR_BLIT) {
        if (t->src_phys == 0 || t->stride == 0 || t->x >= cur.w || t->y >= cur.h) {
            return;
        }
        uint32_t *src = (uint32_t *)phys_to_virt(t->src_phys);
        if (src == NULL) {
            return;
        }
        uint32_t width = t->w < cur.w - t->x ? t->w : cur.w - t->x;
        uint32_t height = t->h < cur.h - t->y ? t->h : cur.h - t->y;
        for (uint32_t y = 0; y < height; y++) {
            for (uint32_t x = 0; x < width; x++) {
                dst[(t->y + y) * stride + t->x + x] = src[y * t->stride + x];
            }
        }
        return;
    }

    if (t->kind == HDMI_TR_FLIP && t->dst_phys != 0) {
        cur.fb_phys = t->dst_phys;
    }
}

uint64_t hdmi_submit(const hdmi_transfer_t *t)
{
    if (t == NULL || !op_ready || pending >= HDMI_MAX_TRANSFERS) {
        return 0;
    }

    if (t->kind != HDMI_TR_FILL && t->kind != HDMI_TR_BLIT && t->kind != HDMI_TR_FLIP) {
        return 0;
    }

    for (uint32_t i = 0; i < HDMI_MAX_TRANSFERS; i++) {
        if (!slot_used[i]) {
            slots[i] = *t;
            slot_seq[i] = next_seq++;
            if (next_seq == 0) {
                next_seq = 1;
            }
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
    if (out_seq == NULL) {
        return false;
    }

    for (uint32_t i = 0; i < HDMI_MAX_TRANSFERS; i++) {
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

uint64_t hdmi_submit_fill(uint32_t color, uint32_t x, uint32_t y, uint32_t w, uint32_t h)
{
    hdmi_transfer_t t = {
        .kind = HDMI_TR_FILL,
        .color = color,
        .x = x,
        .y = y,
        .w = w,
        .h = h,
    };
    return hdmi_submit(&t);
}

void hdmi_op_set_mode(const hdmi_mode_t *m)
{
    if (m == NULL) {
        return;
    }
    cur_mode = *m;
    cur.w = m->w;
    cur.h = m->h;
    cur.ready = (cur.fb_phys != 0 && cur.w != 0 && cur.h != 0 && cur.stride >= cur.w) ? 1 : 0;
    op_ready = cur.ready != 0;
}

static bool fb_granted = false;

bool hdmi_fb_grant(void)
{
    if (cur.fb_phys == 0) {
        return false;
    }

    fb_granted = true;
    return true;
}

void hdmi_fb_revoke(void)
{
    fb_granted = false;
}

bool hdmi_fb_granted(void)
{
    return fb_granted;
}

void hdmi_caps_raw(uint32_t *w, uint32_t *h, uint32_t *s, uint64_t *phys)
{
    if (w == NULL || h == NULL || s == NULL || phys == NULL) {
        return;
    }
    *w = cur.w;
    *h = cur.h;
    *s = cur.stride;
    *phys = cur.fb_phys;
}
