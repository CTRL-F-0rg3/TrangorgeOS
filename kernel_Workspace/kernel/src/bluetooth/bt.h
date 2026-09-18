#ifndef BT_H
#define BT_H

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

bool bt_init(void);
bool bt_ready(void);
void bt_info(uint8_t *hci_ver, uint8_t *bdaddr);
bool bt_hci_cmd(uint16_t opcode, const uint8_t *params, uint8_t len);
bool bt_event_poll(uint8_t *buf, uint8_t cap, uint8_t *len);
bool bt_acl_send(const uint8_t *data, uint16_t len);
bool bt_acl_recv(uint8_t *data, uint16_t cap, uint16_t *len);

#endif
