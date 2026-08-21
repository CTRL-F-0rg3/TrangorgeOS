#ifndef CORELANG_NATIVE_H
#define CORELANG_NATIVE_H

#include "bc.h"
#include "vm.h"

/* signature wygenerowanej funkcji:
   uint64_t fn(uint64_t *slots, uint64_t *globals, cl_ext_fn *externs) */
typedef uint64_t (*cl_native_fn)(uint64_t *, uint64_t *, cl_ext_fn *);

/* Zwraca bufor z kodem (RW) albo NULL gdy arch nie wspiera subsetu.
   Wywołujący robi go executable (mprotect / mapowanie WX w kernelu). */
void *cl_native_compile(cl_prog_t *P);
size_t cl_native_size(void);

#endif