#include "vgpu.h"
#include "dsabi.h"

#define VBE_INDEX 0x01CE
#define VBE_DATA  0x01CF

static vgpu_info_t g_info;
static vgpu_surface_t g_surf[VGPU_MAX_SURFACES];
static bool g_ready = false;

/* ---------- PCI / port helpers ---------- */

static void pci_void(uint32_t op, uint64_t a0, uint64_t a1, uint64_t a2)
{
    uint64_t id = ds_call(SVC_PCI, op, a0, a1, a2);
    ds_poll();
    ds_msg_t r;
    ds_take(id, &r);
}

static uint64_t pci_val(uint32_t op, uint64_t a0, uint64_t a1, uint64_t a2)
{
    uint64_t id = ds_call(SVC_PCI, op, a0, a1, a2);
    ds_poll();
    ds_msg_t r;

    if (ds_take(id, &r) && r.status == 0) {
        return r.arg0;
    }

    return 0;
}

static void vbe_write(uint16_t idx, uint16_t val)
{
    pci_void(PORT_WRITE, VBE_INDEX, idx, 16);
    pci_void(PORT_WRITE, VBE_DATA, val, 16);
}

/* ---------- init ---------- */

bool vgpu_init(uint32_t w, uint32_t h)
{
    if (g_ready) {
        return true;
    }

    uint64_t id = ds_call(SVC_PCI, PCI_FIND, 0x0300, 0, 0);
    ds_poll();

    ds_msg_t r;

    if (!ds_take(id, &r) || r.status != 0) {
        return false;
    }

    uint64_t bdf = r.arg0;

    if (r.arg1 != (0x1234 | (0x1111 << 16))) {
        return false;
    }

    uint64_t fb_phys = pci_val(PCI_BAR, bdf, 0, 0);

    if (fb_phys == 0) {
        return false;
    }

    pci_void(PCI_ENABLE, bdf, 0, 0);

    vbe_write(4, 0);
    vbe_write(0, 0xB0C4);
    vbe_write(1, w);
    vbe_write(2, h);
    vbe_write(3, 32);
    vbe_write(4, 0x01 | 0x40 | 0x80);

    uint64_t mid = ds_call(SVC_SYS, OP_MAPMMIO, fb_phys,
                           (uint64_t)w * h * 4, 0x45000000ULL);
    ds_poll();

    ds_msg_t mr;

    if (!ds_take(mid, &mr) || mr.status != 0) {
        return false;
    }

    g_info.width = w;
    g_info.height = h;
    g_info.bpp = 32;
    g_info.stride = w;
    g_info.fb_phys = fb_phys;
    g_info.fb = (void *)0x45000000ULL;

    g_ready = true;

    return true;
}

vgpu_info_t vgpu_get_info(void)
{
    return g_info;
}

/* ---------- fb ops ---------- */

void vgpu_pixel(uint32_t x, uint32_t y, uint32_t color)
{
    if (!g_ready || x >= g_info.width || y >= g_info.height) {
        return;
    }

    ((uint32_t *)g_info.fb)[y * g_info.stride + x] = color;
}

void vgpu_clear(uint32_t color)
{
    if (!g_ready) {
        return;
    }

    uint32_t *fb = (uint32_t *)g_info.fb;
    uint32_t n = g_info.width * g_info.height;

    for (uint32_t i = 0; i < n; i++) {
        fb[i] = color;
    }
}

/* ---------- surfaces ---------- */

int32_t vgpu_surface_create(uint32_t w, uint32_t h, uint64_t *phys_out)
{
    uint32_t pages = (w * h * 4 + 4095) / 4096;

    uint64_t aid = ds_call(SVC_SYS, OP_ALLOC, pages, 0, 0);
    ds_poll();

    ds_msg_t ar;

    if (!ds_take(aid, &ar) || ar.status == 0 ? ar.arg0 == 0 : 1) {
        return -1;
    }

    uint64_t va = ar.arg0;

    uint64_t pid = ds_call(SVC_SYS, OP_PAGEPHYS, va, 0, 0);
    ds_poll();

    ds_msg_t pr;

    if (!ds_take(pid, &pr) || pr.status != 0) {
        return -1;
    }

    for (int32_t i = 0; i < VGPU_MAX_SURFACES; i++) {
        if (!g_surf[i].used) {
            g_surf[i].used = true;
            g_surf[i].phys = pr.arg0;
            g_surf[i].va = (void *)va;
            g_surf[i].w = w;
            g_surf[i].h = h;

            *phys_out = pr.arg0;
            return i;
        }
    }

    return -1;
}

bool vgpu_surface_present(int32_t id, uint32_t x, uint32_t y)
{
    if (id < 0 || id >= VGPU_MAX_SURFACES || !g_surf[id].used) {
        return false;
    }

    vgpu_surface_t *s = &g_surf[id];
    uint32_t *src = (uint32_t *)s->va;

    for (uint32_t yy = 0; yy < s->h && y + yy < g_info.height; yy++) {
        for (uint32_t xx = 0; xx < s->w && x + xx < g_info.width; xx++) {
            uint32_t c = src[yy * s->w + xx];

            if ((c & 0xFF000000) != 0) {
                ((uint32_t *)g_info.fb)[(y + yy) * g_info.stride + (x + xx)] = c;
            }
        }
    }

    return true;
}

/* ---------- ring server ---------- */

void vgpu_process_ring(volatile vgpu_slot_t *ring)
{
    if (ring == (void *)0) {
        return;
    }

    for (uint32_t i = 0; i < VGPU_RING_SLOTS; i++) {
        volatile vgpu_slot_t *s = &ring[i];

        if (s->id == 0 || s->done != 0) {
            continue;
        }

        switch (s->op) {
        case VGPU_INFO:
            s->r0 = ((uint64_t)g_info.width << 32) | g_info.height;
            s->r1 = g_info.fb_phys;
            s->status = 0;
            break;

        case VGPU_SURF_CREATE: {
            uint64_t phys = 0;
            int32_t sid = vgpu_surface_create((uint32_t)s->a0,
                                              (uint32_t)s->a1, &phys);

            if (sid < 0) {
                s->status = -1;
            } else {
                s->r0 = (uint64_t)(uint32_t)sid;
                s->r1 = phys;
                s->status = 0;
            }
            break;
        }

        case VGPU_PRESENT:
            s->status = vgpu_surface_present((int32_t)s->a0,
                                             (uint32_t)s->a1,
                                             (uint32_t)s->a2) ? 0 : -1;
            break;

        default:
            s->status = -8;
            break;
        }

        s->done = 1;
    }
}

/* ---------- entry ---------- */

static volatile vgpu_slot_t *g_ring = (void *)0;

__attribute__((section(".text.ds_entry"), used))
void ds_entry(uint64_t params_va)
{
    static bool inited = false;

    ds_init(params_va);

    if (!inited) {
        if (vgpu_init(1024, 768)) {
            uint64_t rid = ds_call(SVC_SYS, OP_ALLOC, 1, 0, 0);
            ds_poll();

            ds_msg_t rr;

            if (ds_take(rid, &rr) && rr.status == 0) {
                g_ring = (volatile vgpu_slot_t *)rr.arg0;

                uint64_t pid = ds_call(SVC_SYS, OP_PAGEPHYS, rr.arg0, 0, 0);
                ds_poll();

                ds_msg_t pr;

                if (ds_take(pid, &pr) && pr.status == 0) {
                    ds_call(SVC_VGPU, VGPU_REGISTER, pr.arg0, 0, 0);
                    ds_poll();

                    ds_msg_t zr;
                    ds_take(0, &zr);
                }
            }

            vgpu_clear(0xFF101020);
        }

        inited = true;
    }

    vgpu_process_ring(g_ring);

    ds_yield();
}