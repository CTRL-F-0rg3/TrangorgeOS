CC = clang
CFLAGS_COMMON = -ffreestanding -fno-builtin -fno-stack-protector -mno-red-zone \
                -nostdlib -Iinclude -Wall -Wextra -O2

CFLAGS_KERNEL = $(CFLAGS_COMMON) -DKSTD_KERNEL_BUILD
CFLAGS_USER   = $(CFLAGS_COMMON) -DKSTD_USER_BUILD

# Pliki wspólne (np. string.c, parsery)
COMMON_SRC = src/common/string.c

# Pliki specyficzne dla środowiska
KERNEL_SRC = src/kernel/video.c src/kernel/audio.c src/kernel/bt.c
USER_SRC   = src/user/video.c src/user/audio.c src/user/bt.c

all: libkstd_kernel.a libkstd_user.a

libkstd_kernel.a: $(COMMON_SRC) $(KERNEL_SRC)
	$(CC) $(CFLAGS_KERNEL) -c $^
	ar rcs $@ *.o
	rm *.o

libkstd_user.a: $(COMMON_SRC) $(USER_SRC)
	$(CC) $(CFLAGS_USER) -c $^
	ar rcs $@ *.o
	rm *.o

clean:
	rm -f *.a *.o