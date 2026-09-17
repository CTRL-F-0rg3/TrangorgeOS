proc nim_parse_u64(s: ptr uint8, len: uint32, base: uint8, out_v: ptr uint64): uint8
  {.exportc, cdecl.} =
  var v = 0u64
  var i = 0u32
  let b = uint64(base)

  if len == 0: return 0

  if base == 16 and len > 2 and s[0] == uint8('0') and (s[1] == uint8('x') or s[1] == uint8('X')):
    i = 2

  if i >= len: return 0

  while i < len:
    let c = s[i]
    let d =
      if c >= uint8('0') and c <= uint8('9'): uint64(c - uint8('0'))
      elif c >= uint8('a') and c <= uint8('f'): uint64(c - uint8('a')) + 10
      elif c >= uint8('A') and c <= uint8('F'): uint64(c - uint8('A')) + 10
      else: return 0

    if d >= b: return 0

    v = v * b + d
    i += 1

  out_v[] = v
  return 1