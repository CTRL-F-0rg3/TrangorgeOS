#ifndef BT_OPERATION_H
#define BT_OPERATION_H

#include "bt.h"

void bt_op_model_init(void);
void bt_op_model_run_cmd(uint16_t opcode, const uint8_t *params, uint8_t len);

#endif