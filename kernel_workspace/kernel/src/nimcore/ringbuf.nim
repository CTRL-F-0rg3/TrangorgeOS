type Rb = object
  data: array[1024, uint8]
  head: uint32
  tail: uint32

var key_rb: Rb

proc nim_rb_push(b: uint8): uint8 {.exportc, cdecl.} =
  let next = (key_rb.head + 1) mod 1024
  if next == key_rb.tail: return 0
  key_rb.data[key_rb.head] = b
  key_rb.head = next
  return 1

proc nim_rb_pop(): int32 {.exportc, cdecl.} =
  if key_rb.head == key_rb.tail: return -1
  let b = key_rb.data[key_rb.tail]
  key_rb.tail = (key_rb.tail + 1) mod 1024
  return int32(b)

proc nim_rb_len(): uint32 {.exportc, cdecl.} =
  return (key_rb.head + 1024 - key_rb.tail) mod 1024