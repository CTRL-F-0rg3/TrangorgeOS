package abi

import "util"

MESSAGE_MAGIC :: u32(0x54524E47)

MessageFlag :: bit_set[u32] {
    None,
    Is_Reply,
    Non_Blocking,
    Urgent,
    Kernel_Originated,
}

MessageHeader :: struct #packed {
    magic:      u32,
    sender_pid: util.PID,
    target_pid: util.PID,
    opcode:     Opcode,
    flags:      MessageFlag,
    payload_sz: u32,
    seq_num:    u64,
}

MessageEnvelope :: struct {
    header:  MessageHeader,
    payload: [dynamic]u8,
}

make_request_header :: proc(sender: util.PID, target: util.PID, op: Opcode, payload_size: u32, seq: u64) -> MessageHeader {
    return MessageHeader{
        magic      = MESSAGE_MAGIC,
        sender_pid = sender,
        target_pid = target,
        opcode     = op,
        flags      = {MessageFlag.None},
        payload_sz = payload_size,
        seq_num    = seq,
    }
}