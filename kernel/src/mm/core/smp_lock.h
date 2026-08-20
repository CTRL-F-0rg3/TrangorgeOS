#ifndef MM_CORE_SMP_LOCK_H
#define MM_CORE_SMP_LOCK_H

#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>


typedef struct smp_ticket_lock {
    volatile uint32_t next_ticket;
    volatile uint32_t now_serving;

    /* Identyfikator rdzenia (APIC id) aktualnego posiadacza biletu, albo
     * -1 gdy blokada jest wolna. Modyfikowany wyłącznie przez rdzeń, który
     * faktycznie trzyma bilet, więc odczyt/zapis nie wymaga dodatkowej
     * synchronizacji ponad tę, którą daje sam bilet. */
    volatile int64_t owner_cpu;

    uint32_t depth;
    uint64_t saved_flags;
} smp_ticket_lock_t;

#define SMP_TICKET_LOCK_INIT { 0, 0, -1, 0, 0 }

void smp_lock_init(smp_ticket_lock_t *lock);

/* Bierze blokadę (z irqsave). Bezpieczne do wywołania zagnieżdżonego z
 * tego samego rdzenia, który już ją trzyma. */
void smp_lock_acquire(smp_ticket_lock_t *lock);

/* Zwalnia blokadę. Zwraca false, jeśli wywołane bez odpowiadającego
 * `smp_lock_acquire()` (błąd wywołującego) — nic wtedy nie modyfikuje. */
bool smp_lock_release(smp_ticket_lock_t *lock);

/* Identyfikator bieżącego rdzenia (initial APIC id z CPUID.1:EBX[31:24]).
 * Działa bez żadnej wcześniejszej inicjalizacji podsystemu SMP/per-CPU —
 * dlatego jest tu, a nie w cache/per_cpu.c, które obecnie zakłada 1 CPU. */
int64_t smp_current_cpu_id(void);

#endif