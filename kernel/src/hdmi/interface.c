#include "interface.h"

#include <stdatomic.h>

static _Atomic uint32_t port_owner = 0;

bool hdmi_iface_acquire(uint32_t owner)
{
    if (owner == 0) {
        return false;
    }

    uint32_t expected = 0;
    if (atomic_compare_exchange_strong(&port_owner, &expected, owner)) {
        return true;
    }

    return expected == owner;
}

bool hdmi_iface_release(uint32_t owner)
{
    uint32_t expected = owner;
    return owner != 0 && atomic_compare_exchange_strong(&port_owner, &expected, 0);
}

uint32_t hdmi_iface_owner(void)
{
    return atomic_load(&port_owner);
}

bool hdmi_iface_check(uint32_t owner)
{
    return owner != 0 && atomic_load(&port_owner) == owner;
}

bool hdmi_iface_init(uint32_t owner, uint64_t fb_phys, uint32_t w, uint32_t h, uint32_t stride)
{
    if (owner != 0 && !hdmi_iface_check(owner)) {
        return false;
    }

    return hdmi_init_with(fb_phys, w, h, stride);
}

bool hdmi_iface_ready(void)
{
    return hdmi_ready();
}

bool hdmi_iface_mode_set(uint32_t owner, uint32_t id)
{
    if (!hdmi_iface_check(owner)) {
        return false;
    }

    return hdmi_mode_set_by_id(id);
}

bool hdmi_iface_mode_current(uint32_t *id, uint32_t *w, uint32_t *h, uint32_t *r)
{
    return hdmi_mode_current_raw(id, w, h, r);
}

bool hdmi_iface_mode_at(uint32_t i, uint32_t *id, uint32_t *w, uint32_t *h, uint32_t *r)
{
    return hdmi_mode_at_raw(i, id, w, h, r);
}

uint64_t hdmi_iface_submit_fill(uint32_t owner, uint32_t color, uint32_t x, uint32_t y, uint32_t w, uint32_t h)
{
    if (!hdmi_iface_check(owner)) {
        return 0;
    }

    return hdmi_submit_fill(color, x, y, w, h);
}

bool hdmi_iface_poll(uint32_t owner, uint64_t *out_seq)
{
    if (!hdmi_iface_check(owner)) {
        return false;
    }

    return hdmi_poll(out_seq);
}

void hdmi_iface_caps(uint32_t *w, uint32_t *h, uint32_t *s, uint64_t *phys)
{
    hdmi_caps_raw(w, h, s, phys);
}

bool hdmi_iface_fb_grant(uint32_t owner, uint64_t *phys, uint32_t *w, uint32_t *h, uint32_t *s)
{
    if (!hdmi_iface_check(owner) || phys == NULL || w == NULL || h == NULL || s == NULL) {
        return false;
    }

    uint64_t p;
    uint32_t ww, hh, ss;
    hdmi_caps_raw(&ww, &hh, &ss, &p);

    if (p == 0 || !hdmi_fb_grant()) {
        return false;
    }

    *phys = p;
    *w = ww;
    *h = hh;
    *s = ss;
    return true;
}

bool hdmi_iface_fb_revoke(uint32_t owner)
{
    if (!hdmi_iface_check(owner)) {
        return false;
    }

    hdmi_fb_revoke();
    return true;
}