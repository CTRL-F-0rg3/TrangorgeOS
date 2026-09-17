#ifndef COMPAT_H
#define COMPAT_H

typedef unsigned char   __u8;
typedef unsigned short  __u16;
typedef unsigned int    __u32;
typedef unsigned long long __u64;


#define __user
#define __kernel


#define __attribute__(x)
#define __packed   __attribute__((packed))
#define __aligned(x)  __attribute__((aligned(x)))


#define static inline static


#define EINVAL  22
#define ENOMEM  12
#define EFAULT  14


#define true    1
#define false   0

#endif 