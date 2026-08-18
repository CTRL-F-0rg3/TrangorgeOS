#ifndef DSBLOCK_H
#define DSBLOCK_H
#include "dsabi.h"

static inline uint64_t ds_blk_count(uint64_t disk)
{
    return ds_call(SVC_BLOCK, BLK_COUNT, disk, 0, 0);
}

static inline uint64_t ds_blk_read(uint64_t disk, uint64_t block)
{
    return ds_call(SVC_BLOCK, BLK_READ, disk, block, 0);
}

static inline uint64_t ds_blk_write(uint64_t disk, uint64_t block)
{
    return ds_call(SVC_BLOCK, BLK_WRITE, disk, block, 0);
}

static inline volatile uint8_t *ds_blk_scratch(void)
{
    return (volatile uint8_t *)DS_SCRATCH_VA;
}

#endif