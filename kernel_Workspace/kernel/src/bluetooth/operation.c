#include "operation.h"
#include "hci.h"

static uint8_t evt_q[EVT_Q][EVT_MAX];
static uint8_t evt_len[EVT_Q];
static uint32_t evt_head;
static uint32_t evt_tail;
static uint8_t acl_q[ACL_Q][ACL_MAX];
static uint16_t acl_len[ACL_Q];
static uint32_t acl_head;
static uint32_t acl_tail;
static uint8_t bdaddr[6] = { 0xDE, 0xAD, 0xBE, 0xEF, 0x00, 0x01 };
static uint8_t hci_version = 0x09;
static bool op_ready;

static bool evt_push(const uint8_t *data, uint8_t len)
{
    if (data == NULL || len == 0 || len > EVT_MAX) {
        return false;
    }

    uint32_t next = (evt_head + 1) % EVT_Q;
    if (next == evt_tail) {
        return false;
    }

    for (uint8_t i = 0; i < len; i++) {
        evt_q[evt_head][i] = data[i];
    }

    evt_len[evt_head] = len;
    evt_head = next;
    return true;
}

void bt_op_model_init(void)
{
    evt_head = 0;
    evt_tail = 0;
    acl_head = 0;
    acl_tail = 0;
    op_ready = true;
}

bool bt_op_model_run_cmd(uint16_t opcode, const uint8_t *params, uint8_t len)
{
    uint8_t ev[EVT_MAX];

    if (len != 0 && params == NULL) {
        return false;
    }

    switch (opcode) {
    case HCI_RESET:
        ev[0] = HCI_EVT_PKT;
        ev[1] = EVT_CMD_COMPLETE;
        ev[2] = 4;
        ev[3] = 1;
        ev[4] = opcode & 0xFF;
        ev[5] = opcode >> 8;
        ev[6] = 0;
        return evt_push(ev, 7);

    case HCI_READ_LOCAL_VERSION:
        ev[0] = HCI_EVT_PKT;
        ev[1] = EVT_CMD_COMPLETE;
        ev[2] = 12;
        ev[3] = 1;
        ev[4] = opcode & 0xFF;
        ev[5] = opcode >> 8;
        ev[6] = 0;
        ev[7] = hci_version;
        ev[8] = 0;
        ev[9] = 0;
        ev[10] = 0;
        ev[11] = 0;
        ev[12] = 0;
        ev[13] = 0;
        ev[14] = 0;
        return evt_push(ev, 15);

    case HCI_READ_BD_ADDR:
        ev[0] = HCI_EVT_PKT;
        ev[1] = EVT_CMD_COMPLETE;
        ev[2] = 10;
        ev[3] = 1;
        ev[4] = opcode & 0xFF;
        ev[5] = opcode >> 8;
        ev[6] = 0;
        for (uint8_t i = 0; i < 6; i++) {
            ev[7 + i] = bdaddr[i];
        }
        return evt_push(ev, 13);

    default:
        ev[0] = HCI_EVT_PKT;
        ev[1] = EVT_CMD_COMPLETE;
        ev[2] = 4;
        ev[3] = 1;
        ev[4] = opcode & 0xFF;
        ev[5] = opcode >> 8;
        ev[6] = 0;
        return evt_push(ev, 7);
    }
}

bool bt_hci_cmd(uint16_t opcode, const uint8_t *params, uint8_t len)
{
    if (!op_ready) {
        return false;
    }

    return bt_op_model_run_cmd(opcode, params, len);
}

bool bt_event_poll(uint8_t *buf, uint8_t cap, uint8_t *len)
{
    if (buf == NULL || len == NULL || evt_head == evt_tail) {
        return false;
    }

    uint8_t n = evt_len[evt_tail];
    if (n > cap) {
        return false;
    }

    for (uint8_t i = 0; i < n; i++) {
        buf[i] = evt_q[evt_tail][i];
    }

    *len = n;
    evt_tail = (evt_tail + 1) % EVT_Q;
    return true;
}

bool bt_acl_send(const uint8_t *data, uint16_t len)
{
    if (!op_ready || (len != 0 && data == NULL) || len > ACL_MAX) {
        return false;
    }

    uint32_t next = (acl_head + 1) % ACL_Q;
    if (next == acl_tail) {
        return false;
    }

    for (uint16_t i = 0; i < len; i++) {
        acl_q[acl_head][i] = data[i];
    }

    acl_len[acl_head] = len;
    acl_head = next;
    return true;
}

bool bt_acl_recv(uint8_t *data, uint16_t cap, uint16_t *len)
{
    if (data == NULL || len == NULL || acl_head == acl_tail) {
        return false;
    }

    uint16_t n = acl_len[acl_tail];
    if (n > cap) {
        return false;
    }

    for (uint16_t i = 0; i < n; i++) {
        data[i] = acl_q[acl_tail][i];
    }

    *len = n;
    acl_tail = (acl_tail + 1) % ACL_Q;
    return true;
}
