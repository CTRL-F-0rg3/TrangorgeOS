package audiodriver

import "core:mem"

nam_base: u64
bm_base: u64

Bdl_Entry :: struct {
	addr: u32,
	ctrl: u32,
}

BDL_N :: 8

// mixer (NAM) — 16-bit
nam16 :: proc(off: u32) -> ^u16 {
	return transmute(^u16)(nam_base + u64(off))
}

// bus master — 8/16/32
bm8 :: proc(off: u32) -> ^u8 {
	return transmute(^u8)(bm_base + u64(off))
}

bm16 :: proc(off: u32) -> ^u16 {
	return transmute(^u16)(bm_base + u64(off))
}

bm32 :: proc(off: u32) -> ^u32 {
	return transmute(^u32)(bm_base + u64(off))
}

GLOB_CNT  :: 0x2C
GLOB_STA  :: 0x30

PI_BDBAR :: 0x00
PI_LVI   :: 0x05
PI_SR    :: 0x06
PI_CR    :: 0x0B

PO_BDBAR :: 0x10
PO_LVI   :: 0x15
PO_SR    :: 0x16
PO_PICB  :: 0x18
PO_CR    :: 0x1B

IOC :: 0x8000_0000

@(export)
ad_init :: proc "C" (nam_va: u64, bm_va: u64) -> i32 {
	nam_base = nam_va
	bm_base = bm_va

	// cold reset
	mem.volatile_store(bm32(GLOB_CNT), 0x04000000)

	for _ in 0..<100000 {
		if mem.volatile_load(bm32(GLOB_STA)) & 0x04 != 0 {
			break
		}
	}

	// master volume: unmute
	mem.volatile_store(nam16(0x02), 0x0000)

	// PCM out volume
	mem.volatile_store(nam16(0x18), 0x0808)

	return 0
}

@(export)
ad_stop :: proc "C" () -> i32 {
	mem.volatile_store(bm8(PO_CR), 0)
	mem.volatile_store(bm8(PI_CR), 0)
	return 0
}

@(export)
ad_position :: proc "C" () -> u32 {
	return u32(mem.volatile_load(bm16(PO_PICB)))
}