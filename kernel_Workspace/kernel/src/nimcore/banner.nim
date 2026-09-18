const ART = """
#####  ####.  .###.  #...#  .####  .###.  ####.  .####  #####  .###.  .####
..#..  #...#  #...#  ##..#  #....  #...#  #...#  #....  #....  #...#  #....
..#..  ####.  #####  #.#.#  #..##  #...#  ####.  #..##  ####.  #...#  .###.
..#..  #..#.  #...#  #..##  #...#  #...#  #..#.  #...#  #....  #...#  ....#
..#..  #...#  #...#  #...#  .####  .###.  #...#  .####  #####  .###.  ####.

T r a n g o r g e O S
kernel // ring0 * driverspace ring1 * userspace ring3
rust * c * nim * ada spark * odin
"""

proc nim_banner(buf: ptr uint8, cap: uint32): uint32 {.exportc, cdecl.} =
  var o = 0u32

  for ch in ART:
    if o >= cap:
      break

    buf[o] = uint8(ch)
    o += 1

  return o