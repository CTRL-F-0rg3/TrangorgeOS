#include "bt.h"
#include "hci.h"
#include "operation.h"

static bool bt_is_ready = false;
static uint8_t saved_ver = 0;
static uint8_t saved_addr[6];

static bool wait_cmd_complete(uint16_t opcode, uint8_t *ret, uint8_t ret_len)
{
    uint8_t buf[EVT_MAX];
    uint8_t len = 0;

    for (int i = 0; i < 32; i++) {
        if (!bt_event_poll(buf, &len)) {
            continue;
        }

        if (buf[0] == HCI_EVT_PKT && buf[1] == EVT_CMD_COMPLETE &&
            buf[4] == (opcode & 0xFF) && buf[5] == (opcode >> 8)) {
            for (uint8_t k = 0; k < ret_len; k++) {
                ret[k] = buf[7 + k];
            }

            return true;
        }
    }

    return false;
}

bool bt_init(void)
{
    if (bt_is_ready) {
        return true;
    }

    bt_op_model_init();

    if (!bt_hci_cmd(HCI_RESET, NULL, 0)) {
        return false;
    }

    uint8_t tmp[8];

    if (!wait_cmd_complete(HCI_RESET, tmp, 1)) {
        return false;
    }

    if (!bt_hci_cmd(HCI_READ_LOCAL_VERSION, NULL, 0)) {
        return false;
    }

    if (!wait_cmd_complete(HCI_READ_LOCAL_VERSION, tmp, 1)) {
        return false;
    }

    saved_ver = tmp[0];

    if (!bt_hci_cmd(HCI_READ_BD_ADDR, NULL, 0)) {
        return false;
    }

    if (!wait_cmd_complete(HCI_READ_BD_ADDR, saved_addr, 6)) {
        return false;
    }

    bt_is_ready = true;

    return true;
}

bool bt_ready(void)
{
    return bt_is_ready;
}

void bt_info(uint8_t *hci_ver, uint8_t *bdaddr)
{
    *hci_ver = saved_ver;

    for (int i = 0; i < 6; i++) {
        bdaddr[i] = saved_addr[i];
    }
}