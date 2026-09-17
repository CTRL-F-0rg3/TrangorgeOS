#ifndef KERNEL_EDITOR_MOUSE_H
#define KERNEL_EDITOR_MOUSE_H

#include <stdint.h>

void mouse_init(void);

int mouse_poll(int *dx, int *dy, int *dz, int *buttons);

#endif