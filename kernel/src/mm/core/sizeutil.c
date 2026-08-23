#include "sizeutil.h"

bool size_is_pow2(size_t v)
{
	return v != 0 && (v & (v - 1)) == 0;
}

bool size_round_up_pow2(size_t v, size_t *out)
{
	if (out == NULL) {
	    return false;
	}

	if (v == 0) {
	    *out = 1;
	    return true;
	}

	if (size_is_pow2(v)) {
	    *out = v;
	    return true;
	}

	/* Największa potęga dwójki reprezentowalna w size_t. */
	size_t highest_pow2 = ((size_t)1) << (sizeof(size_t) * 8 - 1);

	if (v > highest_pow2) {
	    return false;
	}

	size_t p = 1;

	while (p < v) {
	    p <<= 1;
	}

	*out = p;

	return true;
}

bool kfree_pages_validate(size_t pages,
	                      size_t page_size,
	                      size_t usable_bytes,
	                      size_t *out_requested_bytes)
{
	if (out_requested_bytes == NULL || page_size == 0) {
	    return false;
	}

	if (pages == 0) {
	    return false;
	}

	size_t requested_bytes;

	if (!size_pages_to_bytes_checked(pages, page_size, &requested_bytes)) {
	    return false;
	}

	if (usable_bytes != 0 && requested_bytes > usable_bytes) {
	    return false;
	}

	*out_requested_bytes = requested_bytes;

	return true;
}

bool size_bytes_to_pages_checked(size_t bytes,
	                             size_t page_size,
	                             size_t *out_pages)
{
	if (out_pages == NULL || page_size == 0) {
	    return false;
	}

	/* bytes + (page_size - 1) nie moze przepelnic size_t. */
	if (bytes > SIZE_MAX - (page_size - 1)) {
	    return false;
	}

	*out_pages = (bytes + (page_size - 1)) / page_size;

	return true;
}

bool size_pages_to_bytes_checked(size_t pages,
	                             size_t page_size,
	                             size_t *out_bytes)
{
	if (out_bytes == NULL || page_size == 0) {
	    return false;
	}

	if (pages > SIZE_MAX / page_size) {
	    return false;
	}

	*out_bytes = pages * page_size;

	return true;
}
