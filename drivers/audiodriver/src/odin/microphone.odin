package audiodriver

import "core:mem"

@(export)
ad_capture :: proc "C" (data_phys: u64, len: u32, bdl_va: u64, bdl_phys: u64) -> i32 {
	bdl := transmute(^Bdl_Entry) bdl_va

	n := 0
	off: u64 = 0

	for off < u64(len) && n < BDL_N {
		chunk: u32 = 4096
		rem := u32(u64(len) - off)

		if rem < chunk {
			chunk = rem
		}

		bdl[n] = Bdl_Entry{
			addr = u32(data_phys + off),
			ctrl = chunk | IOC,
		}

		off += u64(chunk)
		n += 1
	}

	if n == 0 {
		return -1
	}

	// record select: mic
	mem.volatile_store(nam16(0x1A), 0x0000)

	mem.volatile_store(bm8(PI_CR), 0)
	mem.volatile_store(bm32(PI_BDBAR), u32(bdl_phys))
	mem.volatile_store(bm8(PI_LVI), u8(n - 1))
	mem.volatile_store(bm8(PI_CR), 1)

	return 0
}