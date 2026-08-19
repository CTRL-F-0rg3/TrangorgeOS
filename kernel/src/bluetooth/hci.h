#ifndef BT_HCI_H
#define BT_HCI_H

#include <stdint.h>

#define HCI_CMD_PKT 0x01
#define HCI_ACL_PKT 0x02
#define HCI_EVT_PKT 0x04

#define HCI_RESET              0x0C03
#define HCI_READ_LOCAL_VERSION 0x1001
#define HCI_READ_BD_ADDR       0x1009
#define HCI_SET_EVENT_MASK     0x0C01
#define HCI_LE_SET_ADV_ENABLE  0x200A
#define HCI_LE_SET_SCAN_ENABLE 0x200C

#define EVT_CMD_COMPLETE  0x0E
#define EVT_CMD_STATUS    0x0F
#define EVT_CONN_COMPLETE 0x03
#define EVT_LE_META       0x3E

#endif