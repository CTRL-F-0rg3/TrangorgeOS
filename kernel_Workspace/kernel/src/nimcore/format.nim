proc nim_u64_to_str(v: uint64, base: uint8, buf: ptr uint8, cap: uint32): uint32
  {.exportc, cdecl.} =
  var tmp: array[64, uint8]
  var n = 0u32
  var x = v

  let b = if base < 2: 10u64 else: uint64(base)

  if x == 0:
    tmp[0] = uint8('0')
    n = 1
  else:
    while x > 0 and n < 64:
      let d = x mod b
      tmp[n] = if d < 10: uint8('0') + uint8(d)
               else: uint8('a') + uint8(d) - 10
      x = x div b
      n += 1

  if n > cap: n = cap

  var i = 0u32
  while i < n:
    buf[i] = tmp[n - 1 - i]
    i += 1

  return n

proc nim_hex_dump(src: ptr uint8, len: uint32, buf: ptr uint8, cap: uint32): uint32
  {.exportc, cdecl.} =
  const hexd = "0123456789abcdef"
  var o = 0u32
  var i = 0u32

  while i < len and o + 2 < cap:
    let b = src[i]
    buf[o] = uint8(hexd[int(b shr 4)])
    buf[o + 1] = uint8(hexd[int(b and 15)])
    o += 2
    i += 1

  return o