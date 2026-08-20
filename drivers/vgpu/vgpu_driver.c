#include "vgpu.h"
#include "dsabi.h"

#define VBE_INDEX 0x01CE
#define VBE_DATA  0x01CF

#define VBE_ID_ID      0
#define VBE_ID_XRES    1
#define VBE_ID_YRES    2
#define VBE_ID_BPP     3
#define VBE_ID_ENABLE  4

#define VBE_ENABLED   0x01
#define VBE_LFB       0x40
#define VBE_NOCLEAR   0x80

static vgpu_info_t g_info;
static uint64_t g_bdf = 0;
static bool g_ready = false;
static uint32_t g_frame = 0;

static void port_w16(uint16_t port, uint16_t v)
{
    ds_call(SVC_PCI, PORT_WRITE, port, v, 16);
    ds_poll();
    ds_msg_t r;
    ds_take(r.id = 0, &r);
}

static void vbe_write(uint16_t idx, uint16_t val)
{
    ds_call(SVC_PCI, PORT_WRITE, VBE_INDEX, idx, 16);
    ds_poll();
    ds_msg_t r1;
    /* odpowiedź nieistotna — fire-and-forget, ale poll zjedz */
    while (ds_poll_step()) {}

    ds_call(SVC_PCI, PORT_WRITE, VBE_DATA, val, 16);
    ds_poll();
    while (ds_poll_step()) {}
}
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

bool vgpu_init(uint32_t w, uint32_t h)
{
    if (g_ready) {
        return true;
    }

    /* 1. Znajdź VGA: class 0x03, subclass 0x00 */
    uint64_t id = ds_call(SVC_PCI, PCI_FIND, 0x0300, 0, 0);
    ds_poll();

    ds_msg_t r;

    if (!ds_take(id, &r) || r.status != 0) {
        return false;
    }

    g_bdf = r.arg0;

    /* QEMU std vga = 0x1234:0x1111 */
    if (r.arg1 != (0x1234 | (0x1111 << 16))) {
        return false;
    }

    /* 2. BAR0 = framebuffer */
    uint64_t fb_phys = pci_val(PCI_BAR, g_bdf, 0, 0);

    if (fb_phys == 0) {
        return false;
    }

    /* 3. Enable MMIO */
    pci_void(PCI_ENABLE, g_bdf, 0, 0);

    /* 4. Ustaw tryb VBE */
    vbe_write(VBE_ID_ENABLE, 0);
    vbe_write(VBE_ID_ID, 0xB0C4);
    vbe_write(VBE_ID_XRES, w);
    vbe_write(VBE_ID_YRES, h);
    vbe_write(VBE_ID_BPP, 32);
    vbe_write(VBE_ID_ENABLE, VBE_ENABLED | VBE_LFB | VBE_NOCLEAR);

    /* 5. Mapuj FB */
    uint32_t fb_size = w * h * 4;

    uint64_t mid = ds_call(SVC_SYS, OP_MAPMMIO, fb_phys, fb_size, 0x45000000ULL);
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

void vgpu_shutdown(void)
{
    if (!g_ready) {
        return;
    }

    vbe_write(VBE_ID_ENABLE, 0);
    g_ready = false;
}

vgpu_info_t vgpu_get_info(void)
{
    return g_info;
}

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

    for (uint32_t i = 0; i < g_info.width * g_info.height; i++) {
        fb[i] = color;
    }
}

void vgpu_blit(const uint32_t *src, uint32_t x, uint32_t y,
               uint32_t w, uint32_t h)
{
    if (!g_ready || !src) {
        return;
    }

    uint32_t *fb = (uint32_t *)g_info.fb;

    for (uint32_t yy = 0; yy < h && y + yy < g_info.height; yy++) {
        for (uint32_t xx = 0; xx < w && x + xx < g_info.width; xx++) {
            fb[(y + yy) * g_info.stride + (x + xx)] = src[yy * w + xx];
        }
    }
}

/* ---- tick: jeden kwant driverspace ---- */

static void vgpu_tick(void)
{
    if (!g_ready) {
        return;
    }

    uint32_t f = g_frame++;

    vgpu_clear(0xFF101020);

    /* ruchomy prostokąt */
    uint32_t x = (f * 3) % g_info.width;

    for (uint32_t yy = 0; yy < 120; yy++) {
        for (uint32_t xx = 0; xx < 160; xx++) {
            vgpu_pixel(x < g_info.width - 160 ? x : 0,
                       200 + yy,
                       0xFF00AAFF);
        }
    }

    /* pasek statusu */
    for (uint32_t xx = 0; xx < g_info.width; xx++) {
        vgpu_pixel(xx, 0, 0xFF303030);
        vgpu_pixel(xx, 1, 0xFF00FF88);
    }
}

/* ---- entry point driverspace ---- */

__attribute__((section(".text.ds_entry"), used))
void ds_entry(uint64_t params_va)
{
    static bool inited = false;

    ds_init(params_va);

    if (!inited) {
        if (vgpu_init(1024, 768)) {
            ds_call(SVC_SYS, OP_REG_DRIVER, DRIVER_KIND_VIDEO, 0, 0);
            ds_poll();
            ds_msg_t r;
            ds_take(0, &r);
        }

        inited = true;
    }

    vgpu_tick();

    ds_yield();
}