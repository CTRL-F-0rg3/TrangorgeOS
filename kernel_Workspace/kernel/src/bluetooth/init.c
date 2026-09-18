#include "bt.h"
#include "hci.h"
#include "operation.h"

static bool bt_is_ready;
static uint8_t saved_ver;
static uint8_t saved_addr[6];

static bool wait_cmd_complete(uint16_t opcode, uint8_t *ret, uint8_t ret_len)
{
    uint8_t buf[EVT_MAX];
    uint8_t len = 0;

    if (ret_len != 0 && ret == NULL) {
        return false;
    }

    for (uint8_t i = 0; i < 32; i++) {
        if (!bt_event_poll(buf, EVT_MAX, &len)) {
            continue;
        }

        if (len < 7 || buf[0] != HCI_EVT_PKT || buf[1] != EVT_CMD_COMPLETE) {
            continue;
        }

        if (buf[4] != (uint8_t)(opcode & 0xFF) || buf[5] != (uint8_t)(opcode >> 8)) {
            continue;
        }

        if (buf[6] != 0 || len < (uint8_t)(7 + ret_len)) {
            return false;
        }

        for (uint8_t k = 0; k < ret_len; k++) {
            ret[k] = buf[7 + k];
        }

        return true;
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
    if (!wait_cmd_complete(HCI_RESET, tmp, 0)) {
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
    if (hci_ver != NULL) {
        *hci_ver = saved_ver;
    }

    if (bdaddr != NULL) {
        for (uint8_t i = 0; i < 6; i++) {
            bdaddr[i] = saved_addr[i];
        }
    }
}
