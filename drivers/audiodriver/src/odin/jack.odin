package audiodriver

import "core:mem"

PWR_CTRL_STA :: 0x24

@(export)
ad_jack_present :: proc "C" () -> i32 {
	pwr := mem.volatile_load(nam16(PWR_CTRL_STA))

	// bity ready w powerdown status = tor aktywny
	if pwr & 0x000F != 0 {
		return 1
	}

	// QEMU i tak nie zdejmuje jacka — fallback
	return 1
}

@(export)
ad_set_amp :: proc "C" (on: i32) -> i32 {
	if on == 0 {
		mem.volatile_store(nam16(0x02), 0x8000) // mute
	} else {
		mem.volatile_store(nam16(0x02), 0x0000)
	}
	return 0
}