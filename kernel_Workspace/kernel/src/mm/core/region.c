#include "region.h"

region_t region_make(uint64_t start, uint64_t len)
{
	region_t r;

	r.start = start;
	r.end = start + len;

	return r;
}

bool region_valid(region_t r)
{
	return r.end > r.start;
}

uint64_t region_len(region_t r)
{
	if (!region_valid(r)) {
	    return 0;
	}

	return r.end - r.start;
}

bool region_contains(region_t r, uint64_t addr)
{
	return addr >= r.start && addr < r.end;
}

bool region_overlaps(region_t a, region_t b)
{
	return a.start < b.end && b.start < a.end;
}

region_t region_intersect(region_t a, region_t b)
{
	region_t r;

	r.start = a.start > b.start ? a.start : b.start;
	r.end = a.end < b.end ? a.end : b.end;

	return r;
}

size_t region_subtract(region_t r,
	                   region_t cut,
	                   region_t *out,
	                   size_t max_out)
{
	size_t n = 0;

	if (!region_valid(r) || out == NULL) {
	    return 0;
	}

	if (cut.start > r.start && n < max_out) {
	    region_t before;

	    before.start = r.start;
	    before.end = cut.start < r.end ? cut.start : r.end;

	    if (region_valid(before)) {
	        out[n++] = before;
	    }
	}

	if (cut.end < r.end && n < max_out) {
	    region_t after;

	    after.start = cut.end > r.start ? cut.end : r.start;
	    after.end = r.end;

	    if (region_valid(after)) {
	        out[n++] = after;
	    }
	}

	return n;
}