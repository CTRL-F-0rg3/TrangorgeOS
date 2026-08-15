#include "pmm.h"
#include "frame.h"
#include "../../arch/x86_64/memory.h"

extern void kprintf(const char *fmt, ...);

static void pmm_panic(const char *msg) __attribute__((noreturn));
static void pmm_panic(const char *msg)
{
    kprintf("pmm panic: %s\n", msg);

    for (;;) {
        __asm__ volatile("cli; hlt");
    }
}

static bool pmm_initialized = false;

static size_t pmm_lock_depth = 0;
static uint64_t pmm_lock_flags = 0;

static void pmm_lock(void)
{
    uint64_t flags;

    __asm__ volatile(
        "pushfq\n"
        "popq %0\n"
        "cli"
        : "=r"(flags)
        :
        : "memory"
    );

    if (pmm_lock_depth == 0) {
        pmm_lock_flags = flags;
    }

    pmm_lock_depth++;
}

static void pmm_unlock(void)
{
    if (pmm_lock_depth == 0) {
        pmm_panic("pmm_unlock without lock");
    }

    pmm_lock_depth--;

    if (pmm_lock_depth == 0) {
        uint64_t flags = pmm_lock_flags;

        __asm__ volatile(
            "pushq %0\n"
            "popfq"
            :
            : "r"(flags)
            : "memory"
        );
    }
}

#define PMM_SIZE_MAX ((size_t)-1)

static size_t pmm_bytes_to_frames(size_t bytes)
{
    if (bytes == 0) {
        return 0;
    }

    if (bytes > PMM_SIZE_MAX - ARCH_PAGE_SIZE) {
        return 0;
    }

    return (bytes + ARCH_PAGE_SIZE - 1) / ARCH_PAGE_SIZE;
}

static size_t pmm_align_bytes_to_frames(size_t align_bytes)
{
    if (align_bytes == 0) {
        return 1;
    }

    uint64_t aligned = arch_page_align_up((uint64_t)align_bytes);

    if (aligned == UINT64_MAX || aligned < ARCH_PAGE_SIZE) {
        return 1;
    }

    uint64_t frames = aligned / ARCH_PAGE_SIZE;

    if (frames == 0) {
        return 1;
    }

    if (frames > PMM_SIZE_MAX) {
        return PMM_SIZE_MAX;
    }

    return (size_t)frames;
}

static bool pmm_zero_frames(uint64_t start_phys, size_t count)
{
    for (size_t i = 0; i < count; i++) {
        uint64_t phys = start_phys + (uint64_t)i * ARCH_PAGE_SIZE;

        if (!frame_zero(phys)) {
            return false;
        }
    }

    return true;
}

bool pmm_init(void)
{
    pmm_lock();

    if (pmm_initialized) {
        pmm_unlock();
        return true;
    }

    if (!arch_memory_ready()) {
        pmm_unlock();
        kprintf("pmm_init: arch memory not initialized\n");
        return false;
    }

    if (!frame_init_from_memory()) {
        pmm_unlock();
        kprintf("pmm_init: frame allocator initialization failed\n");
        return false;
    }

    pmm_initialized = true;

    pmm_unlock();

    return true;
}

bool pmm_ready(void)
{
    return pmm_initialized;
}

bool pmm_alloc_frame(uint64_t *out_phys)
{
    if (!pmm_initialized || out_phys == NULL) {
        return false;
    }

    pmm_lock();

    frame_t frame = FRAME_INVALID;
    bool ok = frame_alloc(&frame);

    pmm_unlock();

    if (!ok) {
        return false;
    }

    *out_phys = frame;

    return true;
}

bool pmm_alloc_zero_frame(uint64_t *out_phys)
{
    if (!pmm_initialized || out_phys == NULL) {
        return false;
    }

    pmm_lock();

    frame_t frame = FRAME_INVALID;
    bool ok = frame_alloc_zero(&frame);

    pmm_unlock();

    if (!ok) {
        return false;
    }

    *out_phys = frame;

    return true;
}

bool pmm_alloc_frames(size_t count, uint64_t *out_phys)
{
    if (!pmm_initialized || out_phys == NULL || count == 0) {
        return false;
    }

    pmm_lock();

    frame_t frame = FRAME_INVALID;
    bool ok = frame_alloc_contiguous(count, 1, &frame);

    pmm_unlock();

    if (!ok) {
        return false;
    }

    *out_phys = frame;

    return true;
}

bool pmm_alloc_frames_aligned(size_t count,
                              size_t align_frames,
                              uint64_t *out_phys)
{
    if (!pmm_initialized || out_phys == NULL || count == 0) {
        return false;
    }

    pmm_lock();

    frame_t frame = FRAME_INVALID;
    bool ok = frame_alloc_contiguous(count, align_frames, &frame);

    pmm_unlock();

    if (!ok) {
        return false;
    }

    *out_phys = frame;

    return true;
}


bool pmm_alloc_bytes(size_t bytes, uint64_t *out_phys)
{
    size_t frames = pmm_bytes_to_frames(bytes);

    if (frames == 0) {
        return false;
    }

    return pmm_alloc_frames(frames, out_phys);
}

bool pmm_alloc_zero_bytes(size_t bytes, uint64_t *out_phys)
{
    if (!pmm_initialized || out_phys == NULL || bytes == 0) {
        return false;
    }

    size_t frames = pmm_bytes_to_frames(bytes);

    if (frames == 0) {
        return false;
    }

    if (frames == 1) {
        return pmm_alloc_zero_frame(out_phys);
    }

    pmm_lock();

    frame_t frame = FRAME_INVALID;
    bool ok = frame_alloc_contiguous(frames, 1, &frame);

    pmm_unlock();

    if (!ok) {
        return false;
    }

    if (!pmm_zero_frames(frame, frames)) {
        pmm_free_frames(frame, frames);
        return false;
    }

    *out_phys = frame;

    return true;
}

bool pmm_alloc_contiguous_bytes(size_t bytes,
                                size_t align_bytes,
                                uint64_t *out_phys)
{
    if (!pmm_initialized || out_phys == NULL || bytes == 0) {
        return false;
    }

    size_t frames = pmm_bytes_to_frames(bytes);
    size_t align_frames = pmm_align_bytes_to_frames(align_bytes);

    if (frames == 0 || align_frames == 0) {
        return false;
    }

    return pmm_alloc_frames_aligned(frames, align_frames, out_phys);
}

bool pmm_free_frame(uint64_t phys)
{
    if (!pmm_initialized) {
        return false;
    }

    pmm_lock();

    bool ok = frame_free(phys);

    pmm_unlock();

    return ok;
}

bool pmm_free_frames(uint64_t phys, size_t count)
{
    if (!pmm_initialized) {
        return false;
    }

    if (count == 0) {
        return true;
    }

    pmm_lock();

    bool ok = frame_free_contiguous(phys, count);

    pmm_unlock();

    return ok;
}

bool pmm_free_bytes(uint64_t phys, size_t bytes)
{
    if (!pmm_initialized) {
        return false;
    }

    if (bytes == 0) {
        return true;
    }

    size_t frames = pmm_bytes_to_frames(bytes);

    if (frames == 0) {
        return false;
    }

    return pmm_free_frames(phys, frames);
}

size_t pmm_stat_total_frames(void)
{
    if (!pmm_initialized) {
        return 0;
    }

    return frame_total();
}

size_t pmm_stat_free_frames(void)
{
    if (!pmm_initialized) {
        return 0;
    }

    return frame_free_count();
}

size_t pmm_stat_allocated_frames(void)
{
    if (!pmm_initialized) {
        return 0;
    }

    return frame_allocated();
}

uint64_t pmm_stat_total_bytes(void)
{
    if (!pmm_initialized) {
        return 0;
    }

    uint64_t frames = frame_total();

    if (frames > UINT64_MAX / ARCH_PAGE_SIZE) {
        return UINT64_MAX;
    }

    return frames * ARCH_PAGE_SIZE;
}

uint64_t pmm_stat_free_bytes(void)
{
    if (!pmm_initialized) {
        return 0;
    }

    uint64_t frames = frame_free_count();

    if (frames > UINT64_MAX / ARCH_PAGE_SIZE) {
        return UINT64_MAX;
    }

    return frames * ARCH_PAGE_SIZE;
}

bool pmm_alloc_frames_below(size_t count,
                           size_t align_frames,
                           uint64_t max_phys,
                           uint64_t *out_phys)
{
    if (!pmm_initialized || out_phys == NULL || count == 0) {
        return false;
    }

    pmm_lock();

    frame_t frame = FRAME_INVALID;
    bool ok = frame_alloc_below(count, align_frames, max_phys, &frame);

    pmm_unlock();

    if (!ok) {
        return false;
    }

    *out_phys = frame;

    return true;
}

void pmm_dump(void)
{
    if (!pmm_initialized) {
        kprintf("PMM: not initialized\n");
        return;
    }

    uint64_t total_frames = pmm_stat_total_frames();
    uint64_t free_frames = pmm_stat_free_frames();
    uint64_t allocated_frames = pmm_stat_allocated_frames();

    uint64_t total_mb = pmm_stat_total_bytes() >> 20;
    uint64_t free_mb = pmm_stat_free_bytes() >> 20;

    kprintf("PMM:\n");
    kprintf("  total frames:     %llu\n", (unsigned long long)total_frames);
    kprintf("  free frames:      %llu\n", (unsigned long long)free_frames);
    kprintf("  allocated frames: %llu\n", (unsigned long long)allocated_frames);
    kprintf("  total memory:     %llu MiB\n", (unsigned long long)total_mb);
    kprintf("  free memory:      %llu MiB\n", (unsigned long long)free_mb);
}
#ifdef PMM_DEBUG

bool pmm_self_test(void)
{
    if (!pmm_initialized) {
        return false;
    }

    /*
     * Test 1: pojedyncze ramki.
     */
    uint64_t frames[8];

    for (size_t i = 0; i < 8; i++) {
        if (!pmm_alloc_zero_frame(&frames[i])) {
            kprintf("pmm_self_test: single frame alloc failed\n");
            return false;
        }

        if (!arch_is_page_aligned(frames[i])) {
            kprintf("pmm_self_test: frame not aligned\n");
            return false;
        }

        volatile uint64_t *p =
            (volatile uint64_t *)arch_phys_to_virt(frames[i]);

        for (size_t j = 0; j < (ARCH_PAGE_SIZE / sizeof(uint64_t)); j++) {
            if (p[j] != 0) {
                kprintf("pmm_self_test: zero frame not zero\n");
                return false;
            }
        }
    }

    for (size_t i = 0; i < 8; i++) {
        if (!pmm_free_frame(frames[i])) {
            kprintf("pmm_self_test: single frame free failed\n");
            return false;
        }
    }

    /*
     * Test 2: ciągły zakres 16 ramek.
     */
    uint64_t contiguous = PMM_INVALID_FRAME;

    if (!pmm_alloc_frames(16, &contiguous)) {
        kprintf("pmm_self_test: contiguous alloc failed\n");
        return false;
    }

    if (!arch_is_page_aligned(contiguous)) {
        kprintf("pmm_self_test: contiguous not aligned\n");
        return false;
    }

    if (!pmm_free_frames(contiguous, 16)) {
        kprintf("pmm_self_test: contiguous free failed\n");
        return false;
    }

    /*
     * Test 3: wyrównanie do 2 MiB.
     */
    uint64_t aligned2m = PMM_INVALID_FRAME;

    if (!pmm_alloc_frames_aligned(512, 512, &aligned2m)) {
        kprintf("pmm_self_test: 2M aligned alloc failed\n");
        return false;
    }

    uint64_t align2m_mask = (512ULL * ARCH_PAGE_SIZE) - 1;

    if ((aligned2m & align2m_mask) != 0) {
        kprintf("pmm_self_test: 2M alignment failed\n");
        return false;
    }

    if (!pmm_free_frames(aligned2m, 512)) {
        kprintf("pmm_self_test: 2M aligned free failed\n");
        return false;
    }

    /*
     * Test 4: alokacja bajtowa.
     */
    uint64_t bytes_alloc = PMM_INVALID_FRAME;

    if (!pmm_alloc_zero_bytes(10000, &bytes_alloc)) {
        kprintf("pmm_self_test: bytes alloc failed\n");
        return false;
    }

    size_t bytes_frames = pmm_bytes_to_frames(10000);

    volatile uint64_t *p =
        (volatile uint64_t *)arch_phys_to_virt(bytes_alloc);

    size_t words = bytes_frames * (ARCH_PAGE_SIZE / sizeof(uint64_t));

    for (size_t i = 0; i < words; i++) {
        if (p[i] != 0) {
            kprintf("pmm_self_test: bytes alloc not zero\n");
            return false;
        }
    }

    if (!pmm_free_bytes(bytes_alloc, 10000)) {
        kprintf("pmm_self_test: bytes free failed\n");
        return false;
    }

    kprintf("pmm_self_test: OK\n");

    return true;
}

#endif /* PMM_DEBUG */
