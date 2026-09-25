# TrangorgeOS lib-C — build the C bindings and link them against the Rust core.
#
# The Rust core (../lib) is a pure rlib; the C ABI staticlib lives in
# ../lib-c-ffi. The C side only adds thin helpers and the smoke test, then
# links `libtg_comm_ffi.a`.

RUST_CORE  := ../lib-c-ffi
RUST_LIB   := $(RUST_CORE)/target/release/libtg_comm_ffi.a

CC         ?= gcc
CFLAGS     ?= -std=c11 -O2 -Wall -Wextra -Iinclude
AR         ?= ar
BUILD      := build

all: $(BUILD)/libtgcomm_c.a

$(RUST_LIB):
	cd $(RUST_CORE) && cargo build --release

$(BUILD):
	mkdir -p $@

$(BUILD)/tgcomm.o: src/tgcomm.c include/tgcomm.h | $(BUILD)
	$(CC) $(CFLAGS) -c src/tgcomm.c -o $@

$(BUILD)/libtgcomm_c.a: $(BUILD)/tgcomm.o
	$(AR) rcs $@ $^

test: $(RUST_LIB) $(BUILD)/libtgcomm_c.a
	$(CC) $(CFLAGS) test/test.c $(BUILD)/libtgcomm_c.a $(RUST_LIB) -lpthread -ldl -lm -o $(BUILD)/test_bin
	./$(BUILD)/test_bin

clean:
	rm -rf $(BUILD)

.PHONY: all test clean
