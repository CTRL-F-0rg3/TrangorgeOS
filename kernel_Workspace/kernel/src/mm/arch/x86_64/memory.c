#include "memory.h"
#ifdef ARCH_MEMORY_MULTIBOOT2

#define MULTIBOOT2_BOOTLOADER_MAGIC 0x36d76289u

#define MULTIBOOT_TAG_TYPE_END  0
#define MULTIBOOT_TAG_TYPE_MMAP 6

struct multiboot_tag {
	uint32_t type;
	uint32_t size;
};

struct multiboot_mmap_entry {
	uint64_t addr;
	uint64_t len;
	uint32_t type;
	uint32_t zero;
};

struct multiboot_tag_mmap {
	uint32_t type;
	uint32_t size;
	uint32_t entry_size;
	uint32_t entry_version;
	struct multiboot_mmap_entry entries[];
};

void arch_memory_init_multiboot2(uint32_t magic,
	                             uintptr_t mb_info,
	                             uint64_t kernel_phys_start,
	                             uint64_t kernel_phys_end,
	                             uint64_t initrd_phys_start,
	                             uint64_t initrd_phys_end)
{
	if (magic != MULTIBOOT2_BOOTLOADER_MAGIC) {
	    arch_mem_panic("bad multiboot2 magic");
	}

	if (mb_info < 8) {
	    arch_mem_panic("bad multiboot2 info address");
	}

	static arch_raw_mem_entry_t raw[ARCH_MAX_MEM_REGIONS];
	size_t n = 0;

	uint32_t total_size = *(const uint32_t *)mb_info;

	if (total_size < 8) {
	    arch_mem_panic("multiboot2 info too small");
	}

	uintptr_t end_addr = mb_info + total_size;
	uintptr_t tag_addr = (mb_info + 8 + 7) & ~(uintptr_t)7;

	while (tag_addr + 8 <= end_addr) {
	    const struct multiboot_tag *tag =
	        (const struct multiboot_tag *)tag_addr;

	    if (tag->size < 8) {
	        break;
	    }

	    if (tag->type == MULTIBOOT_TAG_TYPE_END) {
	        break;
	    }

	    if (tag->type == MULTIBOOT_TAG_TYPE_MMAP) {
	        const struct multiboot_tag_mmap *mmap =
	            (const struct multiboot_tag_mmap *)tag;

	        uintptr_t entry_addr = (uintptr_t)mmap->entries;
	        uintptr_t mmap_end = tag_addr + tag->size;

	        if (mmap->entry_size < sizeof(struct multiboot_mmap_entry)) {
	            break;
	        }

	        while (entry_addr + sizeof(struct multiboot_mmap_entry) <= mmap_end) {
	            const struct multiboot_mmap_entry *e =
	                (const struct multiboot_mmap_entry *)entry_addr;

	            if (n < ARCH_MAX_MEM_REGIONS) {
	                raw[n].base = e->addr;
	                raw[n].len = e->len;
	                raw[n].type = e->type;
	                n++;
	            }

	            entry_addr += mmap->entry_size;
	        }
	    }

	    tag_addr = (tag_addr + tag->size + 7) & ~(uintptr_t)7;
	}

	arch_memory_init(raw,
	                 n,
	                 kernel_phys_start,
	                 kernel_phys_end,
	                 initrd_phys_start,
	                 initrd_phys_end);
}

#endif 
extern void kprintf(const char *fmt, ...);

static void arch_mem_panic(const char *msg) __attribute__((noreturn));
static void arch_mem_panic(const char *msg)
{
	kprintf("arch/x86_64/memory.c panic: %s\n", msg);

	for (;;) {
	    __asm__ volatile("cli; hlt");
	}
}
#define ARCH_U64_MAX       UINT64_MAX
#define ARCH_TMP_REGIONS   (ARCH_MAX_MEM_REGIONS * 3)

static arch_mem_info_t mem;
static bool mem_ready = false;

static arch_raw_mem_entry_t raw_sorted[ARCH_MAX_MEM_REGIONS];
static arch_mem_region_t tmp_regions[ARCH_TMP_REGIONS];


bool arch_memory_boot_alloc(uint64_t len, uint64_t align, uint64_t *out_base)
{
	if (!mem_ready || out_base == NULL || len == 0) {
	    return false;
	}

	uint64_t base = 0;

	if (!arch_memory_find_usable(len, align, &base)) {
	    return false;
	}

	arch_memory_reserve_range(base, len);

	*out_base = base;
	return true;
}

static uint64_t safe_end(uint64_t base, uint64_t len)
{
	if (len > ARCH_U64_MAX - base) {
	    return ARCH_U64_MAX;
	}

	return base + len;
}

static uint64_t u64_max(uint64_t a, uint64_t b)
{
	return (a > b) ? a : b;
}

static uint64_t u64_min(uint64_t a, uint64_t b)
{
	return (a < b) ? a : b;
}

static uint64_t align_up_page_safe(uint64_t v)
{
	if (v > ARCH_U64_MAX - ARCH_PAGE_MASK) {
	    return ARCH_U64_MAX;
	}

	return (v + ARCH_PAGE_MASK) & ~ARCH_PAGE_MASK;
}

static uint64_t align_down_page(uint64_t v)
{
	return v & ~ARCH_PAGE_MASK;
}

static uint64_t align_up_generic(uint64_t v, uint64_t align)
{
	if (align == 0) {
	    return v;
	}

	uint64_t mask = align - 1;

	if (v > ARCH_U64_MAX - mask) {
	    return ARCH_U64_MAX;
	}

	return (v + mask) & ~mask;
}

static arch_mem_type_t raw_type_to_arch(uint32_t raw_type)
{
	switch (raw_type) {
	case ARCH_RAW_MEM_USABLE:
	    return ARCH_MEM_TYPE_USABLE;

	case ARCH_RAW_MEM_ACPI_RECLAIM:
	    return ARCH_MEM_TYPE_ACPI_RECLAIM;

	case ARCH_RAW_MEM_BOOTLOADER:
	    return ARCH_MEM_TYPE_BOOTLOADER;

	case ARCH_RAW_MEM_BAD:
	    return ARCH_MEM_TYPE_BAD;

	case ARCH_RAW_MEM_RESERVED:
	case ARCH_RAW_MEM_ACPI_NVS:
	default:
	    return ARCH_MEM_TYPE_RESERVED;
	}
}

static void sort_raw_entries(arch_raw_mem_entry_t *entries, size_t count)
{
	for (size_t i = 1; i < count; i++) {
	    arch_raw_mem_entry_t key = entries[i];
	    size_t j = i;

	    while (j > 0 && entries[j - 1].base > key.base) {
	        entries[j] = entries[j - 1];
	        j--;
	    }

	    entries[j] = key;
	}
}


static void map_add(arch_mem_region_t *list,
	                size_t *count,
	                size_t cap,
	                arch_mem_region_t r)
{
	if (r.len == 0) {
	    return;
	}

	if (*count > 0) {
	    arch_mem_region_t *last = &list[*count - 1];
	    uint64_t last_end = safe_end(last->base, last->len);

	    if (r.base < last_end) {
	        uint64_t delta = last_end - r.base;

	        if (r.len <= delta) {
	            return;
	        }

	        r.base = last_end;
	        r.len -= delta;
	    }

	    
	    if (safe_end(last->base, last->len) == r.base &&
	        last->type == r.type) {
	        last->len += r.len;
	        return;
	    }
	}

	if (*count >= cap) {
	    arch_mem_panic("too many memory regions");
	}

	list[(*count)++] = r;
}

static void map_reserve_range(uint64_t base, uint64_t len)
{
	if (len == 0) {
	    return;
	}

	uint64_t end = safe_end(base, len);

	
	base = align_down_page(base);
	end = align_up_page_safe(end);

	if (end <= base) {
	    return;
	}

	size_t tcount = 0;

	for (size_t i = 0; i < mem.count; i++) {
	    arch_mem_region_t r = mem.regions[i];
	    uint64_t r_end = safe_end(r.base, r.len);

	   
	    if (r_end <= base || r.base >= end) {
	        map_add(tmp_regions, &tcount, ARCH_TMP_REGIONS, r);
	        continue;
	    }

	    
	    if (r.base < base) {
	        arch_mem_region_t before;
	        before.base = r.base;
	        before.len = base - r.base;
	        before.type = r.type;

	        map_add(tmp_regions, &tcount, ARCH_TMP_REGIONS, before);
	    }

	    
	    uint64_t ov_base = u64_max(r.base, base);
	    uint64_t ov_end = u64_min(r_end, end);

	    if (ov_end > ov_base) {
	        arch_mem_region_t reserved;
	        reserved.base = ov_base;
	        reserved.len = ov_end - ov_base;
	        reserved.type = ARCH_MEM_TYPE_RESERVED;

	        map_add(tmp_regions, &tcount, ARCH_TMP_REGIONS, reserved);
	    }

	
	    if (r_end > end) {
	        arch_mem_region_t after;
	        after.base = end;
	        after.len = r_end - end;
	        after.type = r.type;

	        map_add(tmp_regions, &tcount, ARCH_TMP_REGIONS, after);
	    }
	}

	if (tcount > ARCH_MAX_MEM_REGIONS) {
	    arch_mem_panic("memory reserve produced too many regions");
	}

	mem.count = tcount;

	for (size_t i = 0; i < tcount; i++) {
	    mem.regions[i] = tmp_regions[i];
	}
}

static void map_align_usable_regions(void)
{
	size_t tcount = 0;

	for (size_t i = 0; i < mem.count; i++) {
	    arch_mem_region_t r = mem.regions[i];
	    uint64_t r_end = safe_end(r.base, r.len);

	    if (r.type != ARCH_MEM_TYPE_USABLE) {
	        map_add(tmp_regions, &tcount, ARCH_TMP_REGIONS, r);
	        continue;
	    }

	    uint64_t aligned_base = align_up_page_safe(r.base);
	    uint64_t aligned_end = align_down_page(r_end);

	    if (aligned_end <= aligned_base) {
	        arch_mem_region_t tiny;
	        tiny.base = r.base;
	        tiny.len = r.len;
	        tiny.type = ARCH_MEM_TYPE_RESERVED;

	        map_add(tmp_regions, &tcount, ARCH_TMP_REGIONS, tiny);
	        continue;
	    }

	    
	    if (r.base < aligned_base) {
	        arch_mem_region_t before;
	        before.base = r.base;
	        before.len = aligned_base - r.base;
	        before.type = ARCH_MEM_TYPE_RESERVED;

	        map_add(tmp_regions, &tcount, ARCH_TMP_REGIONS, before);
	    }

	
	    {
	        arch_mem_region_t usable;
	        usable.base = aligned_base;
	        usable.len = aligned_end - aligned_base;
	        usable.type = ARCH_MEM_TYPE_USABLE;

	        map_add(tmp_regions, &tcount, ARCH_TMP_REGIONS, usable);
	    }

	    
	    if (aligned_end < r_end) {
	        arch_mem_region_t after;
	        after.base = aligned_end;
	        after.len = r_end - aligned_end;
	        after.type = ARCH_MEM_TYPE_RESERVED;

	        map_add(tmp_regions, &tcount, ARCH_TMP_REGIONS, after);
	    }
	}

	if (tcount > ARCH_MAX_MEM_REGIONS) {
	    arch_mem_panic("memory alignment produced too many regions");
	}

	mem.count = tcount;

	for (size_t i = 0; i < tcount; i++) {
	    mem.regions[i] = tmp_regions[i];
	}
}

static void recalc_stats(void)
{
	uint64_t total_usable = 0;
	uint64_t max_address = 0;
	uint64_t max_usable_address = 0;

	for (size_t i = 0; i < mem.count; i++) {
	    const arch_mem_region_t *r = &mem.regions[i];
	    uint64_t end = safe_end(r->base, r->len);

	    if (end > max_address) {
	        max_address = end;
	    }

	    if (r->type == ARCH_MEM_TYPE_USABLE) {
	        if (total_usable > ARCH_U64_MAX - r->len) {
	            total_usable = ARCH_U64_MAX;
	        } else {
	            total_usable += r->len;
	        }

	        if (end > max_usable_address) {
	            max_usable_address = end;
	        }
	    }
	}

	mem.total_usable = total_usable;
	mem.max_address = max_address;
	mem.max_usable_address = max_usable_address;
	mem.direct_map_base = ARCH_DIRECT_MAP_BASE;
}

void arch_memory_init(const arch_raw_mem_entry_t *entries,
	                  size_t count,
	                  uint64_t kernel_phys_start,
	                  uint64_t kernel_phys_end,
	                  uint64_t initrd_phys_start,
	                  uint64_t initrd_phys_end)
{
	if (mem_ready) {
	    arch_mem_panic("arch_memory_init() called twice");
	}

	if (entries == NULL || count == 0) {
	    arch_mem_panic("empty memory map");
	}

	if (count > ARCH_MAX_MEM_REGIONS) {
	    arch_mem_panic("memory map too large");
	}

	mem.count = 0;

	for (size_t i = 0; i < count; i++) {
	    raw_sorted[i] = entries[i];
	}

	sort_raw_entries(raw_sorted, count);

	for (size_t i = 0; i < count; i++) {
	    uint64_t base = raw_sorted[i].base;
	    uint64_t end = safe_end(base, raw_sorted[i].len);

	    if (base >= end) {
	        continue;
	    }

	    if (mem.count > 0) {
	        const arch_mem_region_t *last = &mem.regions[mem.count - 1];
	        uint64_t last_end = safe_end(last->base, last->len);

	        if (base < last_end) {
	            base = last_end;
	        }

	        if (base >= end) {
	            continue;
	        }
	    }

	    arch_mem_region_t r;
	    r.base = base;
	    r.len = end - base;
	    r.type = raw_type_to_arch(raw_sorted[i].type);

	    map_add(mem.regions, &mem.count, ARCH_MAX_MEM_REGIONS, r);
	}

	if (mem.count == 0) {
	    arch_mem_panic("no valid memory regions after parsing");
	}


	map_reserve_range(0, 0x100000);

	if (kernel_phys_end > kernel_phys_start) {
	    map_reserve_range(kernel_phys_start,
	                      kernel_phys_end - kernel_phys_start);
	}

	if (initrd_phys_end > initrd_phys_start) {
	    map_reserve_range(initrd_phys_start,
	                      initrd_phys_end - initrd_phys_start);
	}


	map_align_usable_regions();

	recalc_stats();
	mem_ready = true;
}

bool arch_memory_ready(void)
{
	return mem_ready;
}

const arch_mem_info_t *arch_memory_get(void)
{
	if (!mem_ready) {
	    return NULL;
	}

	return &mem;
}

size_t arch_memory_regions(const arch_mem_region_t **out)
{
	if (out != NULL) {
	    *out = mem.regions;
	}

	return mem.count;
}

uint64_t arch_memory_total_usable(void)
{
	return mem.total_usable;
}

bool arch_memory_range_is_usable(uint64_t base, uint64_t len)
{
	if (!mem_ready || len == 0) {
	    return false;
	}

	if (!arch_is_page_aligned(base) || !arch_is_page_aligned(len)) {
	    return false;
	}

	uint64_t end = safe_end(base, len);

	if (end <= base) {
	    return false;
	}

	for (size_t i = 0; i < mem.count; i++) {
	    const arch_mem_region_t *r = &mem.regions[i];

	    if (r->type != ARCH_MEM_TYPE_USABLE) {
	        continue;
	    }

	    uint64_t r_end = safe_end(r->base, r->len);

	    if (base >= r->base && end <= r_end) {
	        return true;
	    }
	}

	return false;
}

void arch_memory_reserve_range(uint64_t base, uint64_t len)
{
	map_reserve_range(base, len);
	recalc_stats();
}

bool arch_memory_find_usable(uint64_t len,
	                         uint64_t align,
	                         uint64_t *out_base)
{
	if (!mem_ready || out_base == NULL || len == 0) {
	    return false;
	}

	if (!arch_is_page_aligned(len)) {
	    len = arch_page_align_up(len);
	}

	if (len == ARCH_U64_MAX) {
	    return false;
	}

	if (align < ARCH_PAGE_SIZE) {
	    align = ARCH_PAGE_SIZE;
	}


	if ((align & (align - 1)) != 0) {
	    align = ARCH_PAGE_SIZE;
	}

	for (size_t i = 0; i < mem.count; i++) {
	    const arch_mem_region_t *r = &mem.regions[i];

	    if (r->type != ARCH_MEM_TYPE_USABLE) {
	        continue;
	    }

	    uint64_t r_end = safe_end(r->base, r->len);
	    uint64_t candidate = align_up_generic(r->base, align);

	    if (candidate < r->base) {
	        continue;
	    }

	    if (candidate <= r_end && (r_end - candidate) >= len) {
	        *out_base = candidate;
	        return true;
	    }
	}

	return false;
}



static const char *region_type_name(arch_mem_type_t type)
{
	switch (type) {
	case ARCH_MEM_TYPE_USABLE:
	    return "usable";
	case ARCH_MEM_TYPE_RESERVED:
	    return "reserved";
	case ARCH_MEM_TYPE_ACPI_RECLAIM:
	    return "acpi_reclaim";
	case ARCH_MEM_TYPE_BOOTLOADER:
	    return "bootloader";
	case ARCH_MEM_TYPE_BAD:
	    return "bad";
	default:
	    return "unknown";
	}
}

void arch_memory_dump(void)
{
	kprintf("arch memory map: %u regions, usable %llu MiB, max_addr=0x%llx\n",
	        (unsigned int)mem.count,
	        (unsigned long long)(mem.total_usable >> 20),
	        (unsigned long long)mem.max_address);

	for (size_t i = 0; i < mem.count; i++) {
	    const arch_mem_region_t *r = &mem.regions[i];
	    uint64_t end = safe_end(r->base, r->len);

	    kprintf("  [%s] base=0x%016llx end=0x%016llx len=0x%016llx\n",
	            region_type_name(r->type),
	            (unsigned long long)r->base,
	            (unsigned long long)end,
	            (unsigned long long)r->len);
	}
}