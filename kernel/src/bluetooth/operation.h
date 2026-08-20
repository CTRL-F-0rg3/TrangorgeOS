#ifndef BT_OPERATION_H
#define BT_OPERATION_H

#include "bt.h"

#define EVT_Q 16
#define EVT_MAX 64
#define ACL_Q 8
#define ACL_MAX 256

void bt_op_model_init(void);
bool bt_op_model_run_cmd(uint16_t opcode, const uint8_t *params, uint8_t len);

#endif
