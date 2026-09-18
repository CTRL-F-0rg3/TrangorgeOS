#ifndef CORELANG_BRIDGE_H
#define CORELANG_BRIDGE_H

#include "vm.h"

int cl_bridge_init(cl_vm_t *vm, uint8_t ring);

int cl_bridge_add(cl_vm_t *vm, const char *name, cl_ext_fn fn);

#endif