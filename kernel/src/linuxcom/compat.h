#ifndef COMPAT_H
#define COMPAT_H

/* Minimalni stuby typów i makra dla interfejsu trangorge.c - x86_64 */

/* Typy używane przez trangorge.h i trangorge.c */
typedef unsigned char   __u8;
typedef unsigned short  __u16;
typedef unsigned int    __u32;
typedef unsigned long long __u64;

/* Makra adresowania użytkownika */
#define __user
#define __kernel

/* Konstrukcje kernela */
#define __attribute__(x)
#define __packed   __attribute__((packed))
#define __aligned(x)  __attribute__((aligned(x)))

/* Inline i eksporty */
#define static inline static

/* Kody błędów */
#define EINVAL  22
#define ENOMEM  12
#define EFAULT  14

/* Inne potrzebne definicje */
#define true    1
#define false   0

#endif /* COMPAT_H */