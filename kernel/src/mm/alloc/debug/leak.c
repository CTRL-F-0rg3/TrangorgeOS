#include "leak.h"
#include "../../core/smp_lock.h"

extern void kprintf(const char *fmt, ...);

#define LEAK_MAX 2048

typedef struct leak_entry {
	void *ptr;
	size_t size;
	uint64_t caller, alloc_id;
	int64_t cpu_id;
	bool used;
} leak_entry_t;

static leak_entry_t leak_table[LEAK_MAX];

/*
 * Sekcja 5 planu ulepszeń ("Diagnostyka i obserwowalność"): tabela leaków
 * nie miała żadnej synchronizacji — pod SMP dwa rdzenie mogły jednocześnie
 * przejść `if (!leak_table[i].used)` dla tego samego `i` i oba zapisać do
 * TEGO SAMEGO slotu (klasyczny check-then-act race), trwale gubiąc jeden
 * z wpisów (ten sam efekt co przepełnienie tabeli — patrz komentarz przy
 * `leak_track_result_t` w leak.h). Używamy tego samego prymitywu co reszta
 * MM (core/smp_lock.h), zgodnie z ustaloną konwencją tego projektu.
 */
static smp_ticket_lock_t leak_smp_lock = SMP_TICKET_LOCK_INIT;

static uint64_t next_alloc_id = 1;

bool leak_track_id(void *ptr, size_t size, uint64_t caller, uint64_t *out_id) {
	smp_lock_acquire(&leak_smp_lock);
	for (size_t i = 0; i < LEAK_MAX; i++) {
	    if (!leak_table[i].used) {
	        uint64_t id = next_alloc_id++;
	        leak_table[i].ptr = ptr;
	        leak_table[i].size = size;
	        leak_table[i].caller = caller;
	        leak_table[i].alloc_id = id;
	        leak_table[i].cpu_id = smp_current_cpu_id();
	        leak_table[i].used = true;
	        smp_lock_release(&leak_smp_lock);
	        if (out_id != NULL) *out_id = id;
	        return true;
	    }
	}
	smp_lock_release(&leak_smp_lock);
	return false;
}
bool leak_track(void *ptr, size_t size, uint64_t caller) { return leak_track_id(ptr, size, caller, NULL); }
bool leak_untrack(void *ptr, size_t *out_size, uint64_t *out_caller) {
	smp_lock_acquire(&leak_smp_lock);
	for (size_t i = 0; i < LEAK_MAX; i++) {
	    if (leak_table[i].used && leak_table[i].ptr == ptr) {
	        if (out_size != NULL) *out_size = leak_table[i].size;
	        if (out_caller != NULL) *out_caller = leak_table[i].caller;
	        leak_table[i].used = false;
	        smp_lock_release(&leak_smp_lock);
	        return true;
	    }
	}
	smp_lock_release(&leak_smp_lock);
	return false;
}
bool leak_contains(void *ptr) {
	smp_lock_acquire(&leak_smp_lock);
	for (size_t i = 0; i < LEAK_MAX; i++) {
	    if (leak_table[i].used && leak_table[i].ptr == ptr) {
	        smp_lock_release(&leak_smp_lock);
	        return true;
	    }
	}
	smp_lock_release(&leak_smp_lock);
	return false;
}
size_t leak_count(void) {
	smp_lock_acquire(&leak_smp_lock);
	size_t n = 0;
	for (size_t i = 0; i < LEAK_MAX; i++) if (leak_table[i].used) n++;
	smp_lock_release(&leak_smp_lock);
	return n;
}
void leak_dump(void) {
	smp_lock_acquire(&leak_smp_lock);
	size_t n = 0;
	for (size_t i = 0; i < LEAK_MAX; i++) if (leak_table[i].used) n++;
	kprintf("LEAKS: %llu live\n", (unsigned long long)n);
	for (size_t i = 0; i < LEAK_MAX; i++) {
	    if (!leak_table[i].used) continue;
	    kprintf("  leak: %p size %llu caller 0x%llx id %llu cpu %lld\n",
	            leak_table[i].ptr,
	            (unsigned long long)leak_table[i].size,
	            (unsigned long long)leak_table[i].caller,
	            (unsigned long long)leak_table[i].alloc_id,
	            (long long)leak_table[i].cpu_id);
	}
	smp_lock_release(&leak_smp_lock);
}
