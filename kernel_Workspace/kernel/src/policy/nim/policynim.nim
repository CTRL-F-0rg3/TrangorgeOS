type
  Entry = object
    ring, cls, op, dec: uint8

var
  logbuf: array[256, Entry]
  head = 0
  denies = 0u64

proc nim_policy_log(ring, cls, op, dec: uint8) {.exportc, cdecl.} =
  logbuf[head] = Entry(ring: ring, cls: cls, op: op, dec: dec)
  head = (head + 1) mod 256

  if dec == 1:
    denies += 1

proc nim_policy_denies(): uint64 {.exportc, cdecl.} =
  denies

proc nim_policy_get(idx: uint32,
                    out_ring, out_cls, out_op, out_dec: ptr uint8): uint8
  {.exportc, cdecl.} =
  if idx >= 256:
    return 0

  let e = logbuf[idx]

  if out_ring != nil: out_ring[] = e.ring
  if out_cls  != nil: out_cls[]  = e.cls
  if out_op   != nil: out_op[]   = e.op
  if out_dec  != nil: out_dec[]  = e.dec

  return 1