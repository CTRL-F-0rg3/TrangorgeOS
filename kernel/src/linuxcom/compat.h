#ifndef THEORETICAL_LINUX_COMPAT_H
#define THEORETICAL_LINUX_COMPAT_H

/*
 * compat.h
 * Teoretyczna, userspace/kernel-like warstwa abstrakcji dla C11.
 *
 * Ten plik nie udaje prawdziwego kernela. Eksportuje kontrakty, typy,
 * flagi i funkcje, które można podpiąć do własnego runtime'u, kernela,
 * hypervisora, silnika albo systemu embedded.
 */

#include <stddef.h>
#include <stdint.h>
#include <stdbool.h>
#include <stdarg.h>
#include <time.h>

#ifdef __cplusplus
extern "C" {
#endif

#define LC_API
#define LC_VERSION_MAJOR 0
#define LC_VERSION_MINOR 1
#define LC_VERSION_PATCH 0
#define LC_NAME "theoretical-linux-compat"
#define LC_PAGE_SIZE_DEFAULT 4096u
#define LC_MAX_CPUS 256u
#define LC_MAX_NAME 128u
#define LC_INVALID_HANDLE ((uint64_t)0)
#define LC_CONTAINER_OF(ptr,type,member) ((type *)((char *)(ptr)-offsetof(type,member)))
#define LC_ARRAY_SIZE(a) (sizeof(a)/sizeof((a)[0]))

/* ------------------------------ Kody błędów ----------------------------- */
typedef enum lc_status {
    LC_OK = 0,
    LC_EPERM = -1, LC_ENOENT = -2, LC_EIO = -5, LC_ENXIO = -6,
    LC_E2BIG = -7, LC_ENOMEM = -12, LC_EACCES = -13, LC_EFAULT = -14,
    LC_EBUSY = -16, LC_EEXIST = -17, LC_ENODEV = -19, LC_ENOTDIR = -20,
    LC_EISDIR = -21, LC_EINVAL = -22, LC_ENFILE = -23, LC_EMFILE = -24,
    LC_ENOSPC = -28, LC_EROFS = -30, LC_ENAMETOOLONG = -36,
    LC_ENOSYS = -38, LC_ELOOP = -40, LC_ENODATA = -61, LC_ETIMEDOUT = -110,
    LC_ECANCELED = -125, LC_EOVERFLOW = -75, LC_EAGAIN = -11
} lc_status_t;

const char *lc_strerror(lc_status_t status);

/* ------------------------------ Podstawy -------------------------------- */
typedef uint64_t lc_handle_t;
typedef uint64_t lc_phys_addr_t;
typedef uint64_t lc_virt_addr_t;
typedef uint64_t lc_pid_t;
typedef uint32_t lc_cpu_id_t;
typedef uint32_t lc_uid_t;
typedef uint32_t lc_gid_t;
typedef int64_t lc_ssize_t;
typedef int64_t lc_off_t;
typedef uint64_t lc_jiffies_t;

typedef struct lc_uuid { uint8_t b[16]; } lc_uuid_t;
typedef struct lc_timespec { int64_t sec; int32_t nsec; } lc_timespec_t;
typedef struct lc_iovec { void *base; size_t len; } lc_iovec_t;
typedef struct lc_const_iovec { const void *base; size_t len; } lc_const_iovec_t;

/* ------------------------------ Alokator -------------------------------- */
typedef void *(*lc_alloc_fn)(void *ctx, size_t size, size_t align, uint32_t flags);
typedef void (*lc_free_fn)(void *ctx, void *ptr, size_t size, size_t align);
typedef void *(*lc_realloc_fn)(void *ctx, void *ptr, size_t old_size, size_t new_size, size_t align, uint32_t flags);
typedef void *(*lc_map_fn)(void *ctx, size_t size, uint32_t prot, uint32_t flags);
typedef lc_status_t (*lc_unmap_fn)(void *ctx, void *addr, size_t size);

enum { LC_ALLOC_ZERO=1u<<0, LC_ALLOC_ATOMIC=1u<<1, LC_ALLOC_DMA=1u<<2,
       LC_ALLOC_NOSLEEP=1u<<3, LC_ALLOC_PINNED=1u<<4, LC_ALLOC_EXEC=1u<<5 };
enum { LC_PROT_NONE=0, LC_PROT_READ=1u<<0, LC_PROT_WRITE=1u<<1,
       LC_PROT_EXEC=1u<<2, LC_PROT_USER=1u<<3 };
enum { LC_MAP_ANON=1u<<0, LC_MAP_SHARED=1u<<1, LC_MAP_PRIVATE=1u<<2,
       LC_MAP_FIXED=1u<<3, LC_MAP_DEVICE=1u<<4 };

typedef struct lc_allocator {
    void *ctx; lc_alloc_fn alloc; lc_free_fn free; lc_realloc_fn realloc;
    lc_map_fn map; lc_unmap_fn unmap; size_t page_size;
} lc_allocator_t;

/* ------------------------------ Logowanie -------------------------------- */
typedef enum lc_log_level { LC_LOG_EMERG=0, LC_LOG_ALERT, LC_LOG_CRIT,
    LC_LOG_ERR, LC_LOG_WARN, LC_LOG_NOTICE, LC_LOG_INFO, LC_LOG_DEBUG } lc_log_level_t;
typedef void (*lc_log_fn)(void *ctx, lc_log_level_t level, const char *subsystem,
                          const char *file, int line, const char *fmt, va_list ap);
typedef struct lc_logger { void *ctx; lc_log_fn write; lc_log_level_t min_level; } lc_logger_t;

#define LC_LOG(ks,lvl,sub,fmt,...) lc_log((ks),(lvl),(sub),__FILE__,__LINE__,(fmt),##__VA_ARGS__)

/* ---------------------------- Synchronizacja ----------------------------- */
typedef struct lc_spinlock { uintptr_t opaque[2]; } lc_spinlock_t;
typedef struct lc_mutex { uintptr_t opaque[4]; } lc_mutex_t;
typedef struct lc_rwlock { uintptr_t opaque[6]; } lc_rwlock_t;
typedef struct lc_condvar { uintptr_t opaque[4]; } lc_condvar_t;
typedef struct lc_completion { uintptr_t opaque[4]; } lc_completion_t;
typedef struct lc_atomic32 { volatile int32_t value; } lc_atomic32_t;
typedef struct lc_atomic64 { volatile int64_t value; } lc_atomic64_t;

typedef enum lc_lock_flags { LC_LOCK_INTERRUPTIBLE=1u<<0, LC_LOCK_NOWAIT=1u<<1 } lc_lock_flags_t;

/* -------------------------------- Workqueue ------------------------------- */
typedef void (*lc_work_fn)(void *arg);
typedef struct lc_work { lc_work_fn fn; void *arg; uintptr_t opaque[4]; } lc_work_t;
typedef struct lc_workqueue lc_workqueue_t;

typedef struct lc_timer { lc_work_fn fn; void *arg; uint64_t deadline_ns; uintptr_t opaque[4]; } lc_timer_t;

/* -------------------------------- Procesy --------------------------------- */
typedef enum lc_task_state { LC_TASK_NEW, LC_TASK_READY, LC_TASK_RUNNING,
    LC_TASK_SLEEPING, LC_TASK_STOPPED, LC_TASK_ZOMBIE, LC_TASK_DEAD } lc_task_state_t;
typedef void *(*lc_thread_fn)(void *arg);
typedef struct lc_task_attr { const char *name; size_t stack_size; int priority; uint32_t flags; } lc_task_attr_t;

/* -------------------------------- Pliki ---------------------------------- */
typedef struct lc_file lc_file_t;
typedef struct lc_inode lc_inode_t;
typedef struct lc_dirent { uint64_t ino; uint64_t off; uint16_t reclen; uint8_t type; char name[LC_MAX_NAME]; } lc_dirent_t;
typedef struct lc_stat { uint64_t dev, ino, size, blocks; uint32_t mode, nlink, uid, gid; int64_t atime, mtime, ctime; } lc_stat_t;

enum { LC_O_RDONLY=0, LC_O_WRONLY=1, LC_O_RDWR=2, LC_O_CREAT=1u<<6,
       LC_O_EXCL=1u<<7, LC_O_TRUNC=1u<<9, LC_O_APPEND=1u<<10,
       LC_O_NONBLOCK=1u<<11, LC_O_DIRECTORY=1u<<16 };
enum { LC_S_IFREG=0100000u, LC_S_IFDIR=0040000u, LC_S_IFCHR=0020000u,
       LC_S_IFBLK=0060000u, LC_S_IFLNK=0120000u };

/* -------------------------------- Sieć ----------------------------------- */
typedef struct lc_socket lc_socket_t;
typedef struct lc_sockaddr { uint16_t family; uint8_t data[126]; } lc_sockaddr_t;
typedef enum lc_socket_type { LC_SOCK_STREAM=1, LC_SOCK_DGRAM=2, LC_SOCK_RAW=3 } lc_socket_type_t;
typedef enum lc_socket_family { LC_AF_UNSPEC=0, LC_AF_UNIX=1, LC_AF_INET=2, LC_AF_INET6=10 } lc_socket_family_t;

/* -------------------------------- Urządzenia ------------------------------ */
typedef enum lc_device_type { LC_DEV_BLOCK, LC_DEV_CHAR, LC_DEV_NET, LC_DEV_MISC } lc_device_type_t;
typedef struct lc_device lc_device_t;
typedef struct lc_device_ops { lc_status_t (*open)(lc_device_t *); lc_status_t (*close)(lc_device_t *);
    lc_ssize_t (*read)(lc_device_t *, void *, size_t, lc_off_t *);
    lc_ssize_t (*write)(lc_device_t *, const void *, size_t, lc_off_t *);
    lc_status_t (*ioctl)(lc_device_t *, uint64_t, void *); } lc_device_ops_t;

/* ----------------------------- VFS / syscall ----------------------------- */
typedef struct lc_vfs_ops {
    lc_status_t (*open)(void *, const char *, uint32_t, uint32_t, lc_file_t **);
    lc_status_t (*close)(void *, lc_file_t *);
    lc_ssize_t (*read)(void *, lc_file_t *, void *, size_t, lc_off_t *);
    lc_ssize_t (*write)(void *, lc_file_t *, const void *, size_t, lc_off_t *);
    lc_status_t (*stat)(void *, const char *, lc_stat_t *);
    lc_status_t (*mkdir)(void *, const char *, uint32_t);
    lc_status_t (*unlink)(void *, const char *);
    lc_status_t (*rename)(void *, const char *, const char *);
    lc_status_t (*sync)(void *);
} lc_vfs_ops_t;

typedef struct lc_net_ops {
    lc_status_t (*socket)(void *, int, int, int, lc_socket_t **);
    lc_status_t (*close)(void *, lc_socket_t *);
    lc_status_t (*bind)(void *, lc_socket_t *, const lc_sockaddr_t *, size_t);
    lc_status_t (*listen)(void *, lc_socket_t *, int);
    lc_status_t (*connect)(void *, lc_socket_t *, const lc_sockaddr_t *, size_t);
    lc_status_t (*accept)(void *, lc_socket_t *, lc_socket_t **);
    lc_ssize_t (*send)(void *, lc_socket_t *, const void *, size_t, uint32_t);
    lc_ssize_t (*recv)(void *, lc_socket_t *, void *, size_t, uint32_t);
} lc_net_ops_t;

/* ------------------------------ Kernel state ------------------------------ */
typedef struct lc_hooks { lc_allocator_t memory; lc_logger_t logger; lc_vfs_ops_t vfs;
    lc_net_ops_t net; void *user; } lc_hooks_t;
typedef struct lc_kernel_config { size_t page_size; uint32_t cpu_count; uint32_t max_tasks;
    uint32_t max_handles; uint64_t monotonic_hz; bool deterministic_time; } lc_kernel_config_t;
typedef struct lc_kernel lc_kernel_t;

/* ----------------------------- API publiczne ------------------------------ */
LC_API lc_status_t lc_kernel_init(lc_kernel_t **out, const lc_kernel_config_t *cfg, const lc_hooks_t *hooks);
LC_API void lc_kernel_shutdown(lc_kernel_t *k);
LC_API void *lc_alloc(lc_kernel_t *k, size_t size, size_t align, uint32_t flags);
LC_API void lc_free(lc_kernel_t *k, void *ptr, size_t size, size_t align);
LC_API void *lc_zalloc(lc_kernel_t *k, size_t size, size_t align);
LC_API void lc_log(lc_kernel_t *k, lc_log_level_t level, const char *subsystem, const char *file, int line, const char *fmt, ...);
LC_API uint64_t lc_now_ns(lc_kernel_t *k);
LC_API lc_jiffies_t lc_jiffies(lc_kernel_t *k);
LC_API void lc_sleep_ns(lc_kernel_t *k, uint64_t ns);
LC_API unsigned lc_cpu_count(const lc_kernel_t *k);
LC_API unsigned lc_current_cpu(const lc_kernel_t *k);
LC_API uint64_t lc_random_u64(lc_kernel_t *k);

LC_API void lc_spin_init(lc_spinlock_t *l); void lc_spin_lock(lc_spinlock_t *l); bool lc_spin_trylock(lc_spinlock_t *l); void lc_spin_unlock(lc_spinlock_t *l);
LC_API lc_status_t lc_mutex_init(lc_mutex_t *m); void lc_mutex_destroy(lc_mutex_t *m); lc_status_t lc_mutex_lock(lc_mutex_t *m, uint32_t flags); lc_status_t lc_mutex_trylock(lc_mutex_t *m); void lc_mutex_unlock(lc_mutex_t *m);
LC_API lc_status_t lc_rwlock_init(lc_rwlock_t *l); void lc_rwlock_destroy(lc_rwlock_t *l); lc_status_t lc_read_lock(lc_rwlock_t *l); lc_status_t lc_write_lock(lc_rwlock_t *l); void lc_read_unlock(lc_rwlock_t *l); void lc_write_unlock(lc_rwlock_t *l);
LC_API lc_status_t lc_cond_init(lc_condvar_t *c); void lc_cond_destroy(lc_condvar_t *c); lc_status_t lc_cond_wait(lc_condvar_t *c, lc_mutex_t *m, uint64_t timeout_ns); void lc_cond_signal(lc_condvar_t *c); void lc_cond_broadcast(lc_condvar_t *c);
LC_API lc_status_t lc_completion_init(lc_completion_t *c); void lc_complete(lc_completion_t *c); void lc_complete_all(lc_completion_t *c); lc_status_t lc_wait_for_completion(lc_completion_t *c, uint64_t timeout_ns);
LC_API int32_t lc_atomic_read32(const lc_atomic32_t *a); void lc_atomic_set32(lc_atomic32_t *a,int32_t v); int32_t lc_atomic_add_return32(lc_atomic32_t *a,int32_t v); bool lc_atomic_cmpxchg32(lc_atomic32_t *a,int32_t oldv,int32_t newv);
LC_API int64_t lc_atomic_read64(const lc_atomic64_t *a); void lc_atomic_set64(lc_atomic64_t *a,int64_t v); int64_t lc_atomic_add_return64(lc_atomic64_t *a,int64_t v); bool lc_atomic_cmpxchg64(lc_atomic64_t *a,int64_t oldv,int64_t newv);

LC_API lc_status_t lc_thread_create(lc_kernel_t *k, lc_handle_t *out, lc_thread_fn fn, void *arg, const lc_task_attr_t *attr);
LC_API lc_status_t lc_thread_join(lc_kernel_t *k, lc_handle_t h, void **result); LC_API lc_status_t lc_thread_detach(lc_kernel_t *k, lc_handle_t h); LC_API lc_status_t lc_thread_cancel(lc_kernel_t *k, lc_handle_t h); LC_API lc_pid_t lc_current_pid(void);
LC_API lc_status_t lc_workqueue_create(lc_kernel_t *k, const char *name, unsigned workers, lc_workqueue_t **out); LC_API void lc_workqueue_destroy(lc_workqueue_t *q); LC_API lc_status_t lc_queue_work(lc_workqueue_t *q, lc_work_t *w); LC_API lc_status_t lc_flush_workqueue(lc_workqueue_t *q);
LC_API void lc_timer_init(lc_timer_t *t, lc_work_fn fn, void *arg); LC_API lc_status_t lc_timer_start(lc_kernel_t *k, lc_timer_t *t, uint64_t delay_ns); LC_API lc_status_t lc_timer_cancel(lc_timer_t *t);

LC_API lc_status_t lc_file_open(lc_kernel_t *k,const char *path,uint32_t flags,uint32_t mode,lc_file_t **out); LC_API lc_status_t lc_file_close(lc_kernel_t *k,lc_file_t *f); LC_API lc_ssize_t lc_file_read(lc_kernel_t *k,lc_file_t *f,void *buf,size_t n); LC_API lc_ssize_t lc_file_write(lc_kernel_t *k,lc_file_t *f,const void *buf,size_t n); LC_API lc_status_t lc_file_stat(lc_kernel_t *k,const char *path,lc_stat_t *st); LC_API lc_status_t lc_file_sync(lc_kernel_t *k);
LC_API lc_status_t lc_socket_create(lc_kernel_t *k,int family,int type,int protocol,lc_socket_t **out); LC_API lc_status_t lc_socket_close(lc_kernel_t *k,lc_socket_t *s); LC_API lc_status_t lc_socket_bind(lc_kernel_t *k,lc_socket_t *s,const lc_sockaddr_t *a,size_t n); LC_API lc_status_t lc_socket_connect(lc_kernel_t *k,lc_socket_t *s,const lc_sockaddr_t *a,size_t n); LC_API lc_ssize_t lc_socket_send(lc_kernel_t *k,lc_socket_t *s,const void *p,size_t n,uint32_t f); LC_API lc_ssize_t lc_socket_recv(lc_kernel_t *k,lc_socket_t *s,void *p,size_t n,uint32_t f);
LC_API lc_status_t lc_device_register(lc_kernel_t *k,lc_device_t **out,const char *name,lc_device_type_t type,const lc_device_ops_t *ops,void *priv); LC_API lc_status_t lc_device_unregister(lc_kernel_t *k,lc_device_t *d);
LC_API const lc_hooks_t *lc_hooks(const lc_kernel_t *k); LC_API const lc_kernel_config_t *lc_config(const lc_kernel_t *k);

#ifdef __cplusplus
}
#endif
#endif

/* End of compat.h */
