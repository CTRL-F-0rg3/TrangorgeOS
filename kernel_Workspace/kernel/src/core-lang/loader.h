#ifndef CORELANG_LOADER_H
#define CORELANG_LOADER_H

#include "bc.h"
#include "vm.h"

typedef struct {
    cl_prog_t *bc;
    void *native;          /* NULL gdy arch nie wspiera subsetu */
} cl_module_t;

cl_prog_t *cl_compile_source(const char *src, size_t len, arena_t *ar,
                             uint32_t *err_line, const char **err_msg);

size_t cl_save_bc(cl_prog_t *P, uint8_t *buf, size_t cap);
cl_prog_t *cl_load_bc(const uint8_t *buf, size_t len, arena_t *ar);

cl_module_t *cl_load_path(const char *path, arena_t *ar, bool try_native);

#endif