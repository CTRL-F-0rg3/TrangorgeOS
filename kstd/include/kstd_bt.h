#ifndef KSTD_BT_H
#define KSTD_BT_H

#include "kstd_types.h"

tr_status_t tr_bt_info(uint8_t *hci_ver, uint8_t *addr6);
tr_status_t tr_bt_cmd(uint16_t opcode, const void *params, uint8_t len);
tr_status_t tr_bt_evt(void *buf, uint8_t *len);
tr_status_t tr_bt_acl_send(const void *data, uint16_t len);
tr_status_t tr_bt_acl_recv(void *buf, uint16_t *len);

#endif