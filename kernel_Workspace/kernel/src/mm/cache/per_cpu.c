#include "per_cpu.h"

static size_t cpu_count = 1;

bool per_cpu_init(void)
{
	cpu_count = 1;
	return true;
}

size_t per_cpu_count(void)
{
	return cpu_count;
}

size_t per_cpu_id(void)
{
	return 0;
}