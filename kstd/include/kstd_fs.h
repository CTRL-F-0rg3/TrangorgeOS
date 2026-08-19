#ifndef KSTD_FS_H
#define KSTD_FS_H

#include "kstd_types.h"

tr_status_t tr_fs_read(const char *path, void *buf, uint32_t cap, uint32_t *out);
tr_status_t tr_fs_exists(const char *path);

#endif