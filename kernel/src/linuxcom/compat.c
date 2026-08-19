// #define _GNU_SOURCE
// #define _POSIX_C_SOURCE 200809L
// #include "compat.h"
// #include <errno.h>

// #include <stdlib.h>
// #include <string.h>
// #include <unistd.h>
// #include <fcntl.h>
// #include <sys/stat.h>
// #include <sys/socket.h>
// #include <pthread.h>
// #include <sched.h>
// #include <stdatomic.h>
// #include <stdint.h>
// #include <time.h>

// /*
//  * Backend referencyjny. Własny kernel może zastąpić każdą funkcję przez
//  * lc_hooks_t; kod poniżej jest wyłącznie bezpiecznym adapterem POSIX.
//  */
// struct lc_kernel { lc_kernel_config_t cfg; lc_hooks_t hooks; uint64_t seed; pthread_mutex_t registry_lock; };
// struct lc_file { int fd; lc_kernel_t *k; };
// struct lc_socket { int fd; lc_kernel_t *k; };
// struct lc_inode { uint64_t ino; };
// struct lc_device { char name[LC_MAX_NAME]; lc_device_type_t type; lc_device_ops_t ops; void *priv; };
// struct lc_workqueue { char name[LC_MAX_NAME]; unsigned workers; pthread_mutex_t lock; pthread_cond_t cond; bool stopping; lc_work_t **items; size_t count, cap; pthread_t *threads; };

// typedef struct lc_thread_record { pthread_t thread; lc_handle_t id; bool detached; lc_thread_fn fn; void *arg; } lc_thread_record_t;
// static _Atomic uint64_t g_next_handle = 1;
// static _Thread_local lc_pid_t g_tls_pid;
// static _Thread_local unsigned g_tls_cpu;

// static void *default_alloc(void *ctx,size_t n,size_t a,uint32_t flags) {
//     (void)ctx; void *p=NULL; if (!n) n=1; if (a<sizeof(void*)) a=sizeof(void*);
//     if ((a&(a-1))!=0) a=sizeof(void*);
//     if (posix_memalign(&p,a,n)!=0) return NULL;
//     if (flags&LC_ALLOC_ZERO) memset(p,0,n); return p;
// }
// static void default_free(void *ctx,void *p,size_t n,size_t a){(void)ctx;(void)n;(void)a;free(p);}
// static void *default_realloc(void *ctx,void *p,size_t oldn,size_t newn,size_t a,uint32_t flags){
//     void *q=default_alloc(ctx,newn,a,flags); if(q&&p){memcpy(q,p,oldn<newn?oldn:newn);default_free(ctx,p,oldn,a);} return q;
// }
// static void default_log(void *ctx,lc_log_level_t l,const char *s,const char *f,int line,const char *fmt,va_list ap){
//     (void)ctx; static const char *names[]={"EMERG","ALERT","CRIT","ERR","WARN","NOTICE","INFO","DEBUG"};
//     fprintf(stderr,"[%s] %s (%s:%d): ",names[(l<=LC_LOG_DEBUG)?l:LC_LOG_DEBUG],s?s:"kernel",f,line); vfprintf(stderr,fmt,ap); fputc('\n',stderr);
// }
// static lc_status_t map_errno(int e){switch(e){case 0:return LC_OK;case EPERM:return LC_EPERM;case ENOENT:return LC_ENOENT;case ENOMEM:return LC_ENOMEM;case EACCES:return LC_EACCES;case EBUSY:return LC_EBUSY;case EEXIST:return LC_EEXIST;case ENODEV:return LC_ENODEV;case EINVAL:return LC_EINVAL;case ENOSPC:return LC_ENOSPC;case ENOSYS:return LC_ENOSYS;case ETIMEDOUT:return LC_ETIMEDOUT;case EAGAIN:return LC_EAGAIN;default:return LC_EIO;}}
// const char *lc_strerror(lc_status_t s){switch(s){case LC_OK:return "success";case LC_EPERM:return "operation not permitted";case LC_ENOENT:return "not found";case LC_ENOMEM:return "out of memory";case LC_EACCES:return "permission denied";case LC_EBUSY:return "busy";case LC_EEXIST:return "already exists";case LC_EINVAL:return "invalid argument";case LC_ENOSYS:return "not implemented";case LC_ETIMEDOUT:return "timed out";case LC_EAGAIN:return "try again";default:return "compatibility-layer error";}}

// lc_status_t lc_kernel_init(lc_kernel_t **out,const lc_kernel_config_t *cfg,const lc_hooks_t *hooks){
//     if(!out)return LC_EINVAL; lc_kernel_t *k=calloc(1,sizeof(*k)); if(!k)return LC_ENOMEM;
//     k->cfg=(lc_kernel_config_t){.page_size=LC_PAGE_SIZE_DEFAULT,.cpu_count=1,.max_tasks=1024,.max_handles=4096,.monotonic_hz=1000000000ull,.deterministic_time=false};
//     if(cfg)k->cfg=*cfg; if(!k->cfg.page_size)k->cfg.page_size=LC_PAGE_SIZE_DEFAULT; if(!k->cfg.cpu_count)k->cfg.cpu_count=1;
//     if(hooks)k->hooks=*hooks; if(!k->hooks.memory.alloc)k->hooks.memory=(lc_allocator_t){NULL,default_alloc,default_free,default_realloc,NULL,NULL,k->cfg.page_size};
//     if(!k->hooks.logger.write)k->hooks.logger=(lc_logger_t){NULL,default_log,LC_LOG_INFO}; pthread_mutex_init(&k->registry_lock,NULL); k->seed=(uint64_t)time(NULL)^((uintptr_t)k<<17); *out=k; return LC_OK;
// }
// void lc_kernel_shutdown(lc_kernel_t *k){if(!k)return;pthread_mutex_destroy(&k->registry_lock);free(k);}
// void *lc_alloc(lc_kernel_t *k,size_t n,size_t a,uint32_t f){if(!k||!k->hooks.memory.alloc)return NULL;return k->hooks.memory.alloc(k->hooks.memory.ctx,n,a,f);}
// void lc_free(lc_kernel_t *k,void *p,size_t n,size_t a){if(k&&p&&k->hooks.memory.free)k->hooks.memory.free(k->hooks.memory.ctx,p,n,a);}
// void *lc_zalloc(lc_kernel_t *k,size_t n,size_t a){return lc_alloc(k,n,a,LC_ALLOC_ZERO);}
// void lc_log(lc_kernel_t *k,lc_log_level_t l,const char *s,const char *f,int line,const char *fmt,...){if(!k||!k->hooks.logger.write||l>k->hooks.logger.min_level)return;va_list ap;va_start(ap,fmt);k->hooks.logger.write(k->hooks.logger.ctx,l,s,f,line,fmt,ap);va_end(ap);}
// uint64_t lc_now_ns(lc_kernel_t *k){(void)k;struct timespec t;clock_gettime(CLOCK_MONOTONIC,&t);return (uint64_t)t.tv_sec*1000000000ull+(uint64_t)t.tv_nsec;}
// lc_jiffies_t lc_jiffies(lc_kernel_t *k){return lc_now_ns(k)/(k&&k->cfg.monotonic_hz?k->cfg.monotonic_hz:1000000000ull);}
// void lc_sleep_ns(lc_kernel_t *k,uint64_t ns){(void)k;struct timespec t={ns/1000000000ull,(long)(ns%1000000000ull)};nanosleep(&t,NULL);}
// unsigned lc_cpu_count(const lc_kernel_t *k){return k?k->cfg.cpu_count:1;} unsigned lc_current_cpu(const lc_kernel_t *k){(void)k;return g_tls_cpu;}
// uint64_t lc_random_u64(lc_kernel_t *k){uint64_t x=k?k->seed:0x9e3779b97f4a7c15ull;x^=x<<13;x^=x>>7;x^=x<<17;if(k)k->seed=x;return x;}

// static pthread_mutex_t *as_mutex(lc_mutex_t *m){return (pthread_mutex_t *)(uintptr_t)m->opaque[0];}
// static pthread_rwlock_t *as_rw(lc_rwlock_t *l){return (pthread_rwlock_t *)(uintptr_t)l->opaque[0];}
// static pthread_cond_t *as_cond(lc_condvar_t *c){return (pthread_cond_t *)(uintptr_t)c->opaque[0];}
// static pthread_mutex_t *as_comp_lock(lc_completion_t *c){return (pthread_mutex_t *)(uintptr_t)c->opaque[0];}
// static pthread_cond_t *as_comp_cond(lc_completion_t *c){return (pthread_cond_t *)(uintptr_t)c->opaque[1];}
// void lc_spin_init(lc_spinlock_t *l){l->opaque[0]=0;} void lc_spin_lock(lc_spinlock_t *l){while(!__sync_bool_compare_and_swap(&l->opaque[0],0,1))sched_yield();} bool lc_spin_trylock(lc_spinlock_t *l){return __sync_bool_compare_and_swap(&l->opaque[0],0,1);} void lc_spin_unlock(lc_spinlock_t *l){__sync_lock_release(&l->opaque[0]);}
// lc_status_t lc_mutex_init(lc_mutex_t *m){pthread_mutex_t *p=calloc(1,sizeof(*p));if(!p)return LC_ENOMEM;int e=pthread_mutex_init(p,NULL);if(e){free(p);return map_errno(e);}m->opaque[0]=(uintptr_t)p;return LC_OK;}
// void lc_mutex_destroy(lc_mutex_t *m){pthread_mutex_t*p=as_mutex(m);if(p){pthread_mutex_destroy(p);free(p);}m->opaque[0]=0;}
// lc_status_t lc_mutex_lock(lc_mutex_t*m,uint32_t f){if(!m||!as_mutex(m))return LC_EINVAL;int e=(f&LC_LOCK_NOWAIT)?pthread_mutex_trylock(as_mutex(m)):pthread_mutex_lock(as_mutex(m));return map_errno(e);}
// lc_status_t lc_mutex_trylock(lc_mutex_t*m){return lc_mutex_lock(m,LC_LOCK_NOWAIT);}void lc_mutex_unlock(lc_mutex_t*m){if(m&&as_mutex(m))pthread_mutex_unlock(as_mutex(m));}
// lc_status_t lc_rwlock_init(lc_rwlock_t*l){pthread_rwlock_t*p=calloc(1,sizeof(*p));if(!p)return LC_ENOMEM;int e=pthread_rwlock_init(p,NULL);if(e){free(p);return map_errno(e);}l->opaque[0]=(uintptr_t)p;return LC_OK;}void lc_rwlock_destroy(lc_rwlock_t*l){pthread_rwlock_t*p=as_rw(l);if(p){pthread_rwlock_destroy(p);free(p);}l->opaque[0]=0;}
// lc_status_t lc_read_lock(lc_rwlock_t*l){return map_errno(pthread_rwlock_rdlock(as_rw(l)));}lc_status_t lc_write_lock(lc_rwlock_t*l){return map_errno(pthread_rwlock_wrlock(as_rw(l)));}void lc_read_unlock(lc_rwlock_t*l){pthread_rwlock_unlock(as_rw(l));}void lc_write_unlock(lc_rwlock_t*l){pthread_rwlock_unlock(as_rw(l));}
// lc_status_t lc_cond_init(lc_condvar_t*c){pthread_cond_t*p=calloc(1,sizeof(*p));if(!p)return LC_ENOMEM;int e=pthread_cond_init(p,NULL);if(e){free(p);return map_errno(e);}c->opaque[0]=(uintptr_t)p;return LC_OK;}void lc_cond_destroy(lc_condvar_t*c){pthread_cond_t*p=as_cond(c);if(p){pthread_cond_destroy(p);free(p);}c->opaque[0]=0;}
// static int timedwait_cond(pthread_cond_t*c,pthread_mutex_t*m,uint64_t ns){if(ns==(uint64_t)-1)return pthread_cond_wait(c,m);struct timespec t;clock_gettime(CLOCK_REALTIME,&t);t.tv_sec+=(time_t)(ns/1000000000ull);t.tv_nsec+=(long)(ns%1000000000ull);if(t.tv_nsec>=1000000000L){t.tv_sec++;t.tv_nsec-=1000000000L;}return pthread_cond_timedwait(c,m,&t);}
// lc_status_t lc_cond_wait(lc_condvar_t*c,lc_mutex_t*m,uint64_t ns){return map_errno(timedwait_cond(as_cond(c),as_mutex(m),ns));}void lc_cond_signal(lc_condvar_t*c){pthread_cond_signal(as_cond(c));}void lc_cond_broadcast(lc_condvar_t*c){pthread_cond_broadcast(as_cond(c));}
// lc_status_t lc_completion_init(lc_completion_t*c){pthread_mutex_t*m=calloc(1,sizeof(*m));pthread_cond_t*v=calloc(1,sizeof(*v));if(!m||!v){free(m);free(v);return LC_ENOMEM;}pthread_mutex_init(m,NULL);pthread_cond_init(v,NULL);c->opaque[0]=(uintptr_t)m;c->opaque[1]=(uintptr_t)v;c->opaque[2]=0;return LC_OK;}
// void lc_complete(lc_completion_t*c){pthread_mutex_lock(as_comp_lock(c));c->opaque[2]=1;pthread_cond_signal(as_comp_cond(c));pthread_mutex_unlock(as_comp_lock(c));}void lc_complete_all(lc_completion_t*c){lc_complete(c);}lc_status_t lc_wait_for_completion(lc_completion_t*c,uint64_t ns){pthread_mutex_lock(as_comp_lock(c));while(!c->opaque[2]){int e=timedwait_cond(as_comp_cond(c),as_comp_lock(c),ns);if(e){pthread_mutex_unlock(as_comp_lock(c));return map_errno(e);}}pthread_mutex_unlock(as_comp_lock(c));return LC_OK;}
// int32_t lc_atomic_read32(const lc_atomic32_t*a){return atomic_load((_Atomic int32_t*)&a->value);}void lc_atomic_set32(lc_atomic32_t*a,int32_t v){atomic_store((_Atomic int32_t*)&a->value,v);}int32_t lc_atomic_add_return32(lc_atomic32_t*a,int32_t v){return atomic_fetch_add((_Atomic int32_t*)&a->value,v)+v;}bool lc_atomic_cmpxchg32(lc_atomic32_t*a,int32_t o,int32_t n){return atomic_compare_exchange_strong((_Atomic int32_t*)&a->value,&o,n);}
// int64_t lc_atomic_read64(const lc_atomic64_t*a){return atomic_load((_Atomic int64_t*)&a->value);}void lc_atomic_set64(lc_atomic64_t*a,int64_t v){atomic_store((_Atomic int64_t*)&a->value,v);}int64_t lc_atomic_add_return64(lc_atomic64_t*a,int64_t v){return atomic_fetch_add((_Atomic int64_t*)&a->value,v)+v;}bool lc_atomic_cmpxchg64(lc_atomic64_t*a,int64_t o,int64_t n){return atomic_compare_exchange_strong((_Atomic int64_t*)&a->value,&o,n);}

// static void *thread_trampoline(void *arg){lc_thread_record_t*r=arg;g_tls_pid=r->id;void *ret=r->fn(r->arg);free(r);return ret;}
// lc_status_t lc_thread_create(lc_kernel_t*k,lc_handle_t*out,lc_thread_fn fn,void*arg,const lc_task_attr_t*a){(void)k;if(!out||!fn)return LC_EINVAL;lc_thread_record_t*r=calloc(1,sizeof(*r));if(!r)return LC_ENOMEM;r->id=atomic_fetch_add(&g_next_handle,1);r->fn=fn;r->arg=arg;int e=pthread_create(&r->thread,NULL,thread_trampoline,r);if(e){free(r);return map_errno(e);}if(a&&a->name)pthread_setname_np(r->thread,a->name);*out=r->id;return LC_OK;}
// lc_status_t lc_thread_join(lc_kernel_t*k,lc_handle_t h,void**res){(void)k;(void)h;(void)res;return LC_ENOSYS;}lc_status_t lc_thread_detach(lc_kernel_t*k,lc_handle_t h){(void)k;(void)h;return LC_ENOSYS;}lc_status_t lc_thread_cancel(lc_kernel_t*k,lc_handle_t h){(void)k;(void)h;return LC_ENOSYS;}lc_pid_t lc_current_pid(void){return g_tls_pid;}

// static void *worker_main(void *arg){lc_workqueue_t*q=arg;for(;;){pthread_mutex_lock(&q->lock);while(!q->stopping&&!q->count)pthread_cond_wait(&q->cond,&q->lock);if(q->stopping&&!q->count){pthread_mutex_unlock(&q->lock);return NULL;}lc_work_t*w=q->items[0];memmove(q->items,q->items+1,(q->count-1)*sizeof(*q->items));q->count--;pthread_mutex_unlock(&q->lock);if(w&&w->fn)w->fn(w->arg);}}
// lc_status_t lc_workqueue_create(lc_kernel_t*k,const char*n,unsigned workers,lc_workqueue_t**out){(void)k;if(!out)return LC_EINVAL;if(!workers)workers=1;lc_workqueue_t*q=calloc(1,sizeof(*q));if(!q)return LC_ENOMEM;snprintf(q->name,sizeof(q->name),"%s",n?n:"lc-wq");q->workers=workers;q->cap=64;q->items=calloc(q->cap,sizeof(*q->items));q->threads=calloc(workers,sizeof(*q->threads));pthread_mutex_init(&q->lock,NULL);pthread_cond_init(&q->cond,NULL);for(unsigned i=0;i<workers;i++)if(pthread_create(&q->threads[i],NULL,worker_main,q)){q->stopping=true;}*out=q;return LC_OK;}
// void lc_workqueue_destroy(lc_workqueue_t*q){if(!q)return;pthread_mutex_lock(&q->lock);q->stopping=true;pthread_cond_broadcast(&q->cond);pthread_mutex_unlock(&q->lock);for(unsigned i=0;i<q->workers;i++)pthread_join(q->threads[i],NULL);pthread_cond_destroy(&q->cond);pthread_mutex_destroy(&q->lock);free(q->items);free(q->threads);free(q);}
// lc_status_t lc_queue_work(lc_workqueue_t*q,lc_work_t*w){if(!q||!w)return LC_EINVAL;pthread_mutex_lock(&q->lock);if(q->stopping){pthread_mutex_unlock(&q->lock);return LC_EBUSY;}if(q->count==q->cap){size_t nc=q->cap*2;lc_work_t**ni=realloc(q->items,nc*sizeof(*ni));if(!ni){pthread_mutex_unlock(&q->lock);return LC_ENOMEM;}q->items=ni;q->cap=nc;}q->items[q->count++]=w;pthread_cond_signal(&q->cond);pthread_mutex_unlock(&q->lock);return LC_OK;}
// lc_status_t lc_flush_workqueue(lc_workqueue_t*q){if(!q)return LC_EINVAL;for(;;){pthread_mutex_lock(&q->lock);bool empty=q->count==0;pthread_mutex_unlock(&q->lock);if(empty)return LC_OK;sched_yield();}}
// void lc_timer_init(lc_timer_t*t,lc_work_fn fn,void*a){memset(t,0,sizeof(*t));t->fn=fn;t->arg=a;}lc_status_t lc_timer_start(lc_kernel_t*k,lc_timer_t*t,uint64_t d){(void)k;(void)t;(void)d;return LC_ENOSYS;}lc_status_t lc_timer_cancel(lc_timer_t*t){(void)t;return LC_ENOSYS;}

// lc_status_t lc_file_open(lc_kernel_t*k,const char*p,uint32_t f,uint32_t m,lc_file_t**out){if(!k||!p||!out)return LC_EINVAL;if(k->hooks.vfs.open)return k->hooks.vfs.open(k->hooks.user,p,f,m,out);int flags=(int)(f&0xffffu);int fd=open(p,flags,(mode_t)m);if(fd<0)return map_errno(errno);lc_file_t*x=calloc(1,sizeof(*x));if(!x){close(fd);return LC_ENOMEM;}x->fd=fd;x->k=k;*out=x;return LC_OK;}
// lc_status_t lc_file_close(lc_kernel_t*k,lc_file_t*f){if(k&&k->hooks.vfs.close)return k->hooks.vfs.close(k->hooks.user,f);if(!f)return LC_EINVAL;int e=close(f->fd);free(f);return map_errno(e?errno:0);}lc_ssize_t lc_file_read(lc_kernel_t*k,lc_file_t*f,void*b,size_t n){if(k&&k->hooks.vfs.read)return k->hooks.vfs.read(k->hooks.user,f,b,n,NULL);ssize_t x=read(f->fd,b,n);return x<0?map_errno(errno):x;}lc_ssize_t lc_file_write(lc_kernel_t*k,lc_file_t*f,const void*b,size_t n){if(k&&k->hooks.vfs.write)return k->hooks.vfs.write(k->hooks.user,f,b,n,NULL);ssize_t x=write(f->fd,b,n);return x<0?map_errno(errno):x;}
// lc_status_t lc_file_stat(lc_kernel_t*k,const char*p,lc_stat_t*s){if(k&&k->hooks.vfs.stat)return k->hooks.vfs.stat(k->hooks.user,p,s);struct stat st;if(stat(p,&st)<0)return map_errno(errno);s->ino=st.st_ino;s->size=st.st_size;s->mode=st.st_mode;s->nlink=st.st_nlink;s->uid=st.st_uid;s->gid=st.st_gid;s->atime=st.st_atime;s->mtime=st.st_mtime;s->ctime=st.st_ctime;return LC_OK;}lc_status_t lc_file_sync(lc_kernel_t*k){if(k&&k->hooks.vfs.sync)return k->hooks.vfs.sync(k->hooks.user);return LC_OK;}
// lc_status_t lc_socket_create(lc_kernel_t*k,int fam,int typ,int proto,lc_socket_t**out){if(!k||!out)return LC_EINVAL;if(k->hooks.net.socket)return k->hooks.net.socket(k->hooks.user,fam,typ,proto,out);int fd=socket(fam,typ,proto);if(fd<0)return map_errno(errno);lc_socket_t*s=calloc(1,sizeof(*s));if(!s){close(fd);return LC_ENOMEM;}s->fd=fd;s->k=k;*out=s;return LC_OK;}
// lc_status_t lc_socket_close(lc_kernel_t*k,lc_socket_t*s){if(k&&k->hooks.net.close)return k->hooks.net.close(k->hooks.user,s);if(!s)return LC_EINVAL;int e=close(s->fd);free(s);return map_errno(e?errno:0);}lc_status_t lc_socket_bind(lc_kernel_t*k,lc_socket_t*s,const lc_sockaddr_t*a,size_t n){if(k&&k->hooks.net.bind)return k->hooks.net.bind(k->hooks.user,s,a,n);return bind(s->fd,(const struct sockaddr*)a,(socklen_t)n)<0?map_errno(errno):LC_OK;}lc_status_t lc_socket_connect(lc_kernel_t*k,lc_socket_t*s,const lc_sockaddr_t*a,size_t n){if(k&&k->hooks.net.connect)return k->hooks.net.connect(k->hooks.user,s,a,n);return connect(s->fd,(const struct sockaddr*)a,(socklen_t)n)<0?map_errno(errno):LC_OK;}lc_ssize_t lc_socket_send(lc_kernel_t*k,lc_socket_t*s,const void*p,size_t n,uint32_t f){if(k&&k->hooks.net.send)return k->hooks.net.send(k->hooks.user,s,p,n,f);ssize_t x=send(s->fd,p,n,(int)f);return x<0?map_errno(errno):x;}lc_ssize_t lc_socket_recv(lc_kernel_t*k,lc_socket_t*s,void*p,size_t n,uint32_t f){if(k&&k->hooks.net.recv)return k->hooks.net.recv(k->hooks.user,s,p,n,f);ssize_t x=recv(s->fd,p,n,(int)f);return x<0?map_errno(errno):x;}
// lc_status_t lc_device_register(lc_kernel_t*k,lc_device_t**out,const char*n,lc_device_type_t t,const lc_device_ops_t*o,void*p){(void)k;if(!out||!n)return LC_EINVAL;lc_device_t*d=calloc(1,sizeof(*d));if(!d)return LC_ENOMEM;snprintf(d->name,sizeof(d->name),"%s",n);d->type=t;if(o)d->ops=*o;d->priv=p;*out=d;return LC_OK;}lc_status_t lc_device_unregister(lc_kernel_t*k,lc_device_t*d){(void)k;if(!d)return LC_EINVAL;free(d);return LC_OK;}
// const lc_hooks_t *lc_hooks(const lc_kernel_t*k){return k?&k->hooks:NULL;}const lc_kernel_config_t *lc_config(const lc_kernel_t*k){return k?&k->cfg:NULL;}

// /* End of compat.c */
