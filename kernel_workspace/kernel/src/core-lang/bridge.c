#include "bridge.h"

extern void kprintf(const char *fmt, ...);
extern int32_t k_input_key(void);
extern int32_t k_fs_read(const char *path, void *buf, uint32_t cap);
extern uint32_t k_getpid(void);
extern uint64_t k_world_cr3(void);
extern uint64_t k_kernel_cr3(void);
extern int64_t k_spawn(const char *path, uint32_t parent, uint64_t cr3);
extern int32_t k_ipc_send(uint32_t dst, uint64_t a0, uint64_t a1);
extern int32_t k_ipc_recv(uint64_t *a0, uint64_t *a1);
extern uint64_t k_tick(void);

extern uint64_t hdmi_submit_fill(uint32_t color, uint32_t x, uint32_t y,
                                 uint32_t w, uint32_t h);
extern bool battery_status_packed(uint64_t *a0, uint64_t *a1, uint64_t *a2);

static cl_vm_t *g_vm = (void *)0;
static uint8_t g_ring = 0;


static uint64_t br_put(uint64_t v, uint64_t b, uint64_t c,
                       uint64_t d, uint64_t e, uint64_t f)
{
    (void)b; (void)c; (void)d; (void)e; (void)f;
    kprintf("%llu\n", (unsigned long long)v);
    return 0;
}

static uint64_t br_putc(uint64_t v, uint64_t b, uint64_t c,
                        uint64_t d, uint64_t e, uint64_t f)
{
    (void)b; (void)c; (void)d; (void)e; (void)f;
    kprintf("%c", (char)v);
    return 0;
}

static uint64_t br_log(uint64_t ptr, uint64_t b, uint64_t c,
                       uint64_t d, uint64_t e, uint64_t f)
{
    (void)b; (void)c; (void)d; (void)e; (void)f;

    char buf[128];

    if (g_ring == 0) {
        const char *s = (const char *)ptr;
        int i = 0;

        while (s[i] && i < 127) {
            buf[i] = s[i];
            i++;
        }

        buf[i] = '\0';
    } else {
        extern bool k_user_cstr(uint64_t ptr, char *buf, uint32_t cap);

        if (!k_user_cstr(ptr, buf, sizeof(buf))) {
            return (uint64_t)-1;
        }
    }

    kprintf("%s\n", buf);
    return 0;
}

static uint64_t br_key(uint64_t a, uint64_t b, uint64_t c,
                       uint64_t d, uint64_t e, uint64_t f)
{
    (void)a; (void)b; (void)c; (void)d; (void)e; (void)f;

    int32_t k = k_input_key();

    return k < 0 ? 0 : (uint64_t)k;
}

static uint64_t br_getpid(uint64_t a, uint64_t b, uint64_t c,
                          uint64_t d, uint64_t e, uint64_t f)
{
    (void)a; (void)b; (void)c; (void)d; (void)e; (void)f;
    return k_getpid();
}

static uint64_t br_tick(uint64_t a, uint64_t b, uint64_t c,
                        uint64_t d, uint64_t e, uint64_t f)
{
    (void)a; (void)b; (void)c; (void)d; (void)e; (void)f;
    return k_tick();
}

static uint64_t br_exit(uint64_t code, uint64_t b, uint64_t c,
                        uint64_t d, uint64_t e, uint64_t f)
{
    (void)b; (void)c; (void)d; (void)e; (void)f;

    if (g_vm != (void *)0) {
        g_vm->halt_req = true;
    }

    return code;
}

static uint64_t br_spawn(uint64_t ptr, uint64_t b, uint64_t c,
                         uint64_t d, uint64_t e, uint64_t f)
{
    (void)b; (void)c; (void)d; (void)e; (void)f;

    if (g_ring >= 3) {
        return (uint64_t)-1;
    }

    uint64_t cr3 = (g_ring == 0) ? k_kernel_cr3() : k_world_cr3();

    return (uint64_t)k_spawn((const char *)ptr, k_getpid(), cr3);
}

static uint64_t br_ipc_send(uint64_t dst, uint64_t a0, uint64_t a1,
                            uint64_t d, uint64_t e, uint64_t f)
{
    (void)d; (void)e; (void)f;
    return (uint64_t)(int64_t)k_ipc_send((uint32_t)dst, a0, a1);
}

static uint64_t br_ipc_recv(uint64_t a, uint64_t b, uint64_t c,
                            uint64_t d, uint64_t e, uint64_t f)
{
    (void)a; (void)b; (void)c; (void)d; (void)e; (void)f;

    uint64_t a0 = 0, a1 = 0;
    int32_t from = k_ipc_recv(&a0, &a1);

    return (uint64_t)(int64_t)from;
}

static uint64_t br_fsread(uint64_t path, uint64_t buf, uint64_t cap,
                          uint64_t d, uint64_t e, uint64_t f)
{
    (void)d; (void)e; (void)f;

    if (g_ring >= 3) {
        return (uint64_t)-1;
    }

    return (uint64_t)(int64_t)k_fs_read((const char *)path,
                                        (void *)buf, (uint32_t)cap);
}

static uint64_t br_vfill(uint64_t color, uint64_t xy, uint64_t wh,
                         uint64_t d, uint64_t e, uint64_t f)
{
    (void)d; (void)e; (void)f;

    if (g_ring >= 3) {
        return (uint64_t)-1;
    }

    return hdmi_submit_fill((uint32_t)color,
                            (uint32_t)(xy & 0xFFFF),
                            (uint32_t)(xy >> 16),
                            (uint32_t)(wh & 0xFFFF),
                            (uint32_t)(wh >> 16));
}

static uint64_t br_bat(uint64_t a, uint64_t b, uint64_t c,
                       uint64_t d, uint64_t e, uint64_t f)
{
    (void)a; (void)b; (void)c; (void)d; (void)e; (void)f;

    uint64_t a0 = 0, a1 = 0, a2 = 0;

    if (!battery_status_packed(&a0, &a1, &a2)) {
        return (uint64_t)-1;
    }

    return a0;   
}


int cl_bridge_add(cl_vm_t *vm, const char *name, cl_ext_fn fn)
{
    return cl_vm_register_extern(vm, name, fn);
}

int cl_bridge_init(cl_vm_t *vm, uint8_t ring)
{
    g_vm = vm;
    g_ring = ring;

    cl_bridge_add(vm, "put", br_put);
    cl_bridge_add(vm, "putc", br_putc);
    cl_bridge_add(vm, "log", br_log);
    cl_bridge_add(vm, "key", br_key);
    cl_bridge_add(vm, "tick", br_tick);
    cl_bridge_add(vm, "bat", br_bat);
    cl_bridge_add(vm, "exit", br_exit);

    if (ring <= 1) {
        cl_bridge_add(vm, "getpid", br_getpid);
        cl_bridge_add(vm, "spawn", br_spawn);
        cl_bridge_add(vm, "ipc_send", br_ipc_send);
        cl_bridge_add(vm, "ipc_recv", br_ipc_recv);
        cl_bridge_add(vm, "fsread", br_fsread);
        cl_bridge_add(vm, "vfill", br_vfill);
    }

    return 0;
}