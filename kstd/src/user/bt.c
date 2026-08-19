#include "kstd_bt.h"
#include "dsabi.h"

extern uint64_t ds_call(uint32_t cls, uint32_t op,
                        uint64_t a0, uint64_t a1, uint64_t a2);
extern void ds_poll(void);
extern int ds_take(uint64_t id, ds_msg_t *out);

static volatile uint8_t *scratch = (volatile uint8_t *)DS_SCRATCH_VA;

tr_status_t tr_bt_info(uint8_t *hci_ver, uint8_t *addr6)
{
    uint64_t id = ds_call(SVC_BT, BT_INFO, 0, 0, 0);
    ds_poll();

    ds_msg_t r;

    if (!ds_take(id, &r) || r.status != 0) {
        return TR_ERR_IO;
    }

    *hci_ver = (uint8_t)r.arg0;

    for (int i = 0; i < 6; i++) {
        addr6[i] = (uint8_t)(r.arg1 >> (i * 8));
    }

    return TR_OK;
}

tr_status_t tr_bt_cmd(uint16_t opcode, const void *params, uint8_t len)
{
    const uint8_t *p = (const uint8_t *)params;

    for (uint8_t i = 0; i < len && i < 64; i++) {
        scratch[i] = p[i];
    }

    uint64_t id = ds_call(SVC_BT, BT_CMD, opcode, len, 0);
    ds_poll();

    ds_msg_t r;

    if (ds_take(id, &r) && r.status == 0) {
        return TR_OK;
    }

    return TR_ERR_DENIED;
}

tr_status_t tr_bt_evt(void *buf, uint8_t *len)
{
    uint64_t id = ds_call(SVC_BT, BT_EVT, 0, 0, 0);
    ds_poll();

    ds_msg_t r;

    if (!ds_take(id, &r) || r.status != 0) {
        return TR_ERR_TIMEOUT;
    }

    uint8_t n = (uint8_t)r.arg0;

    uint8_t *dst = (uint8_t *)buf;

    for (uint8_t i = 0; i < n; i++) {
        dst[i] = scratch[i];
    }

    *len = n;
    return TR_OK;
}

tr_status_t tr_bt_acl_send(const void *data, uint16_t len)
{
    const uint8_t *p = (const uint8_t *)data;

    for (uint16_t i = 0; i < len && i < 256; i++) {
        scratch[i] = p[i];
    }

    uint64_t id = ds_call(SVC_BT, BT_ACL_OUT, len, 0, 0);
    ds_poll();

    ds_msg_t r;

    if (ds_take(id, &r) && r.status == 0) {
        return TR_OK;
    }

    return TR_ERR_BUSY;
}

tr_status_t tr_bt_acl_recv(void *buf, uint16_t *len)
{
    uint64_t id = ds_call(SVC_BT, BT_ACL_IN, 0, 0, 0);
    ds_poll();

    ds_msg_t r;

    if (!ds_take(id, &r) || r.status != 0) {
        return TR_ERR_TIMEOUT;
    }

    uint16_t n = (uint16_t)r.arg0;

    uint8_t *dst = (uint8_t *)buf;

    for (uint16_t i = 0; i < n; i++) {
        dst[i] = scratch[i];
    }

    *len = n;
    return TR_OK;
}