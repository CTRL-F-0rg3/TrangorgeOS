// comlimine/src/limine.h
// Minimalny nagłówek protokołu Limine

#ifndef _LIMINE_H
#define _LIMINE_H

#include <stdint.h>
#include <stddef.h>

#define LIMINE_REQUESTS_DELIMITER 0xadc0e0531bb10d03
#define LIMINE_BASE_REVISION(n) uint64_t limine_base_revision[n] = {0x0};

// Magic numbers dla requestów
#define LIMINE_BOOTLOADER_INFO_REQUEST { 0xf55038d8e2a1202f, 0x279426fcf5f59740 }
#define LIMINE_FRAMEBUFFER_REQUEST     { 0x9d5827dcd881dd75, 0xa3148604f6fab11b }
#define LIMINE_MEMMAP_REQUEST          { 0x67cf3d9d378a806f, 0xe304acdfc50c3c62 }
#define LIMINE_HHDM_REQUEST            { 0x48dcf1cb8ad2b852, 0x63984e959a98244b }
#define LIMINE_KERNEL_ADDRESS_REQUEST  { 0x71ba76863cc55f63, 0xb2644a48c516a487 }

// Struktura framebuffer
struct limine_framebuffer {
    void *address;
    uint64_t width;
    uint64_t height;
    uint64_t pitch;
    uint16_t bpp;
    uint8_t memory_model;
    uint8_t red_mask_size;
    uint8_t red_mask_shift;
    uint8_t green_mask_size;
    uint8_t green_mask_shift;
    uint8_t blue_mask_size;
    uint8_t blue_mask_shift;
    uint8_t unused[7];
    uint64_t edid_size;
    void *edid;
    uint64_t mode_count;
    struct limine_video_mode **modes;
};

struct limine_video_mode {
    uint64_t pitch;
    uint64_t width;
    uint64_t height;
    uint16_t bpp;
    uint8_t memory_model;
    uint8_t red_mask_size;
    uint8_t red_mask_shift;
    uint8_t green_mask_size;
    uint8_t green_mask_shift;
    uint8_t blue_mask_size;
    uint8_t blue_mask_shift;
};

// Bootloader info
struct limine_bootloader_info_response {
    uint64_t revision;
    const char *name;
    const char *version;
};

struct limine_bootloader_info_request {
    uint64_t id[2];
    uint64_t revision;
    struct limine_bootloader_info_response *response;
};

// Framebuffer
struct limine_framebuffer_response {
    uint64_t revision;
    uint64_t framebuffer_count;
    struct limine_framebuffer **framebuffers;
};

struct limine_framebuffer_request {
    uint64_t id[2];
    uint64_t revision;
    struct limine_framebuffer_response *response;
};

// Memmap
struct limine_memmap_entry {
    uint64_t base;
    uint64_t length;
    uint32_t type;
};

struct limine_memmap_response {
    uint64_t revision;
    uint64_t entry_count;
    struct limine_memmap_entry **entries;
};

struct limine_memmap_request {
    uint64_t id[2];
    uint64_t revision;
    struct limine_memmap_response *response;
};

// HHDM
struct limine_hhdm_response {
    uint64_t revision;
    uint64_t offset;
};

struct limine_hhdm_request {
    uint64_t id[2];
    uint64_t revision;
    struct limine_hhdm_response *response;
};

// Kernel address
struct limine_kernel_address_response {
    uint64_t revision;
    uint64_t physical_base;
    uint64_t virtual_base;
};

struct limine_kernel_address_request {
    uint64_t id[2];
    uint64_t revision;
    struct limine_kernel_address_response *response;
};

#endif