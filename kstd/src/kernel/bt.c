#include "kstd_bt.h"

extern bool bt_ready(void);
extern void bt_info(uint8_t *ver, uint8_t *addr);
extern bool bt_hci_cmd(uint16_t op, const uint8_t *p, uint8_t len);
extern bool bt_event_poll(uint8_t *buf, uint8_t *len);
extern bool bt_acl_send(const uint8_t *d, uint16_t len);
extern bool bt_acl_recv(uint8_t *d, uint16_t *len);

tr_status_t tr_bt_info(uint8_t *hci_ver, uint8_t *addr6)
{
    if (!bt_ready()) {
        return TR_ERR_IO;
    }

    bt_info(hci_ver, addr6);
    return TR_OK;
}

tr_status_t tr_bt_cmd(uint16_t opcode, const void *params, uint8_t len)
{
    return bt_hci_cmd(opcode, params, len) ? TR_OK : TR_ERR_IO;
}

tr_status_t tr_bt_evt(void *buf, uint8_t *len)
{
    return bt_event_poll(buf, len) ? TR_OK : TR_ERR_TIMEOUT;
}

tr_status_t tr_bt_acl_send(const void *data, uint16_t len)
{
    return bt_acl_send(data, len) ? TR_OK : TR_ERR_BUSY;
}

tr_status_t tr_bt_acl_recv(void *buf, uint16_t *len)
{
    return bt_acl_recv(buf, len) ? TR_OK : TR_ERR_TIMEOUT;
}