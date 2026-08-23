#include "stats.h"
#include "../../core/smp_lock.h"

extern void kprintf(const char *fmt, ...);

static size_t stat_live_count = 0;
static size_t stat_live_bytes = 0;
static size_t stat_peak_bytes = 0;
static size_t stat_total_allocs = 0;
static size_t stat_total_frees = 0;

/*
 * Sekcja 5 planu ulepszeń: liczniki nie miały żadnej synchronizacji —
 * pod SMP dwa rdzenie wołające jednocześnie note_alloc()/note_free()
 * mogły się prześcignąć w `stat_live_count++` (odczyt-modyfikacja-zapis
 * bez atomowości), gubiąc inkrementacje/dekrementacje i psując
 * statystyki w sposób trudny do odróżnienia od prawdziwego wycieku.
 */
static smp_ticket_lock_t stats_smp_lock = SMP_TICKET_LOCK_INIT;

void alloc_stats_note_alloc(size_t size)
{
	smp_lock_acquire(&stats_smp_lock);

	stat_live_count++;
	stat_live_bytes += size;
	stat_total_allocs++;

	if (stat_live_bytes > stat_peak_bytes) {
	    stat_peak_bytes = stat_live_bytes;
	}

	smp_lock_release(&stats_smp_lock);
}

void alloc_stats_note_free(size_t size)
{
	smp_lock_acquire(&stats_smp_lock);

	if (stat_live_count > 0) {
	    stat_live_count--;
	}

	if (stat_live_bytes >= size) {
	    stat_live_bytes -= size;
	} else {
	    stat_live_bytes = 0;
	}

	stat_total_frees++;

	smp_lock_release(&stats_smp_lock);
}

size_t alloc_stats_live_count(void)
{
	smp_lock_acquire(&stats_smp_lock);
	size_t v = stat_live_count;
	smp_lock_release(&stats_smp_lock);
	return v;
}

size_t alloc_stats_live_bytes(void)
{
	smp_lock_acquire(&stats_smp_lock);
	size_t v = stat_live_bytes;
	smp_lock_release(&stats_smp_lock);
	return v;
}

size_t alloc_stats_peak_bytes(void)
{
	smp_lock_acquire(&stats_smp_lock);
	size_t v = stat_peak_bytes;
	smp_lock_release(&stats_smp_lock);
	return v;
}

size_t alloc_stats_total_allocs(void)
{
	smp_lock_acquire(&stats_smp_lock);
	size_t v = stat_total_allocs;
	smp_lock_release(&stats_smp_lock);
	return v;
}

size_t alloc_stats_total_frees(void)
{
	smp_lock_acquire(&stats_smp_lock);
	size_t v = stat_total_frees;
	smp_lock_release(&stats_smp_lock);
	return v;
}

void alloc_stats_dump(void)
{
	smp_lock_acquire(&stats_smp_lock);
	size_t live_count = stat_live_count;
	size_t live_bytes = stat_live_bytes;
	size_t peak_bytes = stat_peak_bytes;
	size_t total_allocs = stat_total_allocs;
	size_t total_frees = stat_total_frees;
	smp_lock_release(&stats_smp_lock);

	kprintf("ALLOC STATS:\n");
	kprintf("  live: %llu objs, %llu B\n",
	        (unsigned long long)live_count,
	        (unsigned long long)live_bytes);
	kprintf("  peak: %llu B\n",
	        (unsigned long long)peak_bytes);
	kprintf("  total allocs: %llu\n",
	        (unsigned long long)total_allocs);
	kprintf("  total frees: %llu\n",
	        (unsigned long long)total_frees);
}
