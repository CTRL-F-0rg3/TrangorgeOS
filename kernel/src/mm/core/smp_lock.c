#include "smp_lock.h"

static inline void cpu_pause(void)
{
    __asm__ volatile("pause" ::: "memory");
}

static inline uint64_t irq_save_disable(void)
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

    return flags;
}

static inline void irq_restore(uint64_t flags)
{
    __asm__ volatile(
        "pushq %0\n"
        "popfq"
        :
        : "r"(flags)
        : "memory"
    );
}

int64_t smp_current_cpu_id(void)
{
    uint32_t eax_out, ebx_out, ecx_out, edx_out;

    __asm__ volatile(
        "cpuid"
        : "=a"(eax_out), "=b"(ebx_out), "=c"(ecx_out), "=d"(edx_out)
        : "a"(1)
        :
    );

    return (int64_t)((ebx_out >> 24) & 0xFFu);
}

void smp_lock_init(smp_ticket_lock_t *lock)
{
    lock->next_ticket = 0;
    lock->now_serving = 0;
    lock->owner_cpu = -1;
    lock->depth = 0;
    lock->saved_flags = 0;
}

void smp_lock_acquire(smp_ticket_lock_t *lock)
{
    uint64_t flags = irq_save_disable();
    int64_t me = smp_current_cpu_id();

    if (lock->depth > 0 &&
        __atomic_load_n(&lock->owner_cpu, __ATOMIC_ACQUIRE) == me) {
        lock->depth++;
        return;
    }

    uint32_t ticket = __atomic_fetch_add(&lock->next_ticket, 1,
                                         __ATOMIC_RELAXED);

    while (__atomic_load_n(&lock->now_serving, __ATOMIC_ACQUIRE) != ticket) {
        cpu_pause();
    }
    lock->owner_cpu = me;
    lock->depth = 1;
    lock->saved_flags = flags;
}

bool smp_lock_release(smp_ticket_lock_t *lock)
{
    if (lock->depth == 0) {
        return false;
    }

    lock->depth--;

    if (lock->depth > 0) {
        return true;
    }

    uint64_t flags = lock->saved_flags;

    lock->owner_cpu = -1;

    __atomic_fetch_add(&lock->now_serving, 1, __ATOMIC_RELEASE);

    irq_restore(flags);

    return true;
}