// comlimine/src/main.c
#include <stdint.h>
#include <stddef.h>
#include "limine.h"

extern void kernel_main(uint64_t magic, void *info);

LIMINE_BASE_REVISION(1);

static volatile struct limine_framebuffer_request framebuffer = {
    .id = LIMINE_FRAMEBUFFER_REQUEST,
    .revision = 0,
    .response = NULL,
};

static volatile struct limine_memmap_request memmap = {
    .id = LIMINE_MEMMAP_REQUEST,
    .revision = 0,
    .response = NULL,
};

static volatile struct limine_hhdm_request hhdm = {
    .id = LIMINE_HHDM_REQUEST,
    .revision = 0,
    .response = NULL,
};

void _start(void) {
    if (framebuffer.response == NULL ||
        memmap.response == NULL ||
        hhdm.response == NULL) {
        for (;;) { asm volatile ("hlt"); }
    }

    kernel_main(0, (void *)framebuffer.response);

    for (;;) { asm volatile ("hlt"); }
}