#include "kstd_fs.h"

extern int32_t k_fs_read(const char *path, void *buf, uint32_t cap);
extern int32_t k_fs_exists(const char *path);

tr_status_t tr_fs_read(const char *path, void *buf, uint32_t cap, uint32_t *out)
{
    int32_t n = k_fs_read(path, buf, cap);

    if (n < 0) {
        return TR_ERR_NOTFOUND;
    }

    if (out) {
        *out = (uint32_t)n;
    }

    return TR_OK;
}

tr_status_t tr_fs_exists(const char *path)
{
    return k_fs_exists(path) == 1 ? TR_OK : TR_ERR_NOTFOUND;
}