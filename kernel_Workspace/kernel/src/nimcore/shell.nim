type Handler = proc(arg: ptr uint8, len: uint32): int32 {.cdecl.}

type Cmd = object
  name: array[24, uint8]
  nlen: uint32
  used: bool
  h: Handler

var cmds: array[32, Cmd]

proc nim_shell_register(name: ptr uint8, nlen: uint32, h: Handler): uint8
  {.exportc, cdecl.} =
  if nlen == 0 or nlen >= 24: return 0

  for i in 0..<32:
    if not cmds[i].used:
      var k = 0u32
      while k < nlen:
        cmds[i].name[k] = name[k]
        k += 1
      cmds[i].nlen = nlen
      cmds[i].used = true
      cmds[i].h = h
      return 1

  return 0

proc eq_name(c: var Cmd, line: ptr uint8, nlen: uint32): bool =
  if c.nlen != nlen: return false
  var k = 0u32
  while k < nlen:
    if c.name[k] != line[k]: return false
    k += 1
  return true

proc nim_shell_run(line: ptr uint8, len: uint32): int32
  {.exportc, cdecl.} =
  var sp = 0u32
  while sp < len and line[sp] != uint8(' '):
    sp += 1

  if sp == 0: return -1

  var arg: ptr uint8 = nil
  var alen = 0u32

  if sp < len:
    arg = cast[ptr uint8](cast[uint](line) + uint(sp + 1))
    alen = len - sp - 1

  for i in 0..<32:
    if cmds[i].used and eq_name(cmds[i], line, sp):
      return cmds[i].h(arg, alen)

  return -2