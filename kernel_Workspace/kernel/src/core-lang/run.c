#include "loader.h"
#include "bridge.h"

extern long cl_kernel_read(const char *path, uint8_t *buf, size_t cap);
extern void *cl_native_compile(cl_prog_t *P);

int cl_run_script(const char *path, uint8_t ring, arena_t *ar)
{
    static uint8_t io[64 * 1024];

    long n = cl_kernel_read(path, io, sizeof(io));

    if (n <= 0) {
        return -1;
    }

    uint32_t el = 0;
    const char *em = (void *)0;

    cl_prog_t *P = cl_compile_source((const char *)io, (size_t)n, ar, &el, &em);

    if (P == (void *)0) {
        extern void kprintf(const char *, ...);
        kprintf("core-lang: %s line %u: %s\n", path, el, em);
        return -1;
    }

    cl_vm_t vm;

    cl_vm_init(&vm, P);
    cl_bridge_init(&vm, ring);

    void *code = cl_native_compile(P);

    if (code != (void *)0) {
        extern void cl_make_exec(void *p, size_t len);
        cl_make_exec(code, 16384);

        static uint64_t slots[64], globs[64];
        static cl_ext_fn exts[16];

        for (size_t i = 0; i < 64; i++) {
            globs[i] = 0;

            for (int k = 0; k < 8; k++) {
                globs[i] |= (uint64_t)P->ginit[i].b[k] << (k * 8);
            }
        }


        for (size_t i = 0; i < P->nexts; i++) {
            exts[i] = vm.externs[i];
        }

        ((void (*)(uint64_t *, uint64_t *, cl_ext_fn *))code)(slots, globs, exts);
    } else {
        cl_vm_run(&vm);
    }

    return 0;
}