#ifndef KERNEL_EDITOR_MOUSE_H
#define KERNEL_EDITOR_MOUSE_H

#include <stdint.h>

void mouse_init(void);

/* Zwraca 1 gdy przyszła paczka. dz = kółko (dodatnie = w dół). */
int mouse_poll(int *dx, int *dy, int *dz, int *buttons);

#endif