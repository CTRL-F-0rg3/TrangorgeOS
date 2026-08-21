#ifndef CORELANG_BRIDGE_H
#define CORELANG_BRIDGE_H

#include "vm.h"

/* Rejestruje externy dozwolone dla danego ringu.
   ring 0: wszystko; ring 1: bez spawn; ring 3: tylko put/putc/log/key/tick/bat. */
int cl_bridge_init(cl_vm_t *vm, uint8_t ring);

int cl_bridge_add(cl_vm_t *vm, const char *name, cl_ext_fn fn);

#endif