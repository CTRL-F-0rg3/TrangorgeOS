#ifndef MM_CORE_SMP_LOCK_H
#define MM_CORE_SMP_LOCK_H

#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>


typedef struct smp_ticket_lock {
    volatile uint32_t next_ticket;
    volatile uint32_t now_serving;

    volatile int64_t owner_cpu;

    uint32_t depth;
    uint64_t saved_flags;
} smp_ticket_lock_t;

#define SMP_TICKET_LOCK_INIT { 0, 0, -1, 0, 0 }

void smp_lock_init(smp_ticket_lock_t *lock);


void smp_lock_acquire(smp_ticket_lock_t *lock);


bool smp_lock_release(smp_ticket_lock_t *lock);


int64_t smp_current_cpu_id(void);

#endif