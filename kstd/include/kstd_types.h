#ifndef KSTD_TYPES_H
#define KSTD_TYPES_H

#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>

/* Prefiks funkcji: tr_ (Trangorge) */

typedef int32_t tr_status_t;

#define TR_OK             0
#define TR_ERR_IO        -1
#define TR_ERR_NOMEM     -2
#define TR_ERR_DENIED    -3  // Polityka SPARK / Autoryzacja odrzuciła
#define TR_ERR_NOTFOUND  -4
#define TR_ERR_INVALID   -5
#define TR_ERR_TIMEOUT   -6
#define TR_ERR_BUSY      -7
#define TR_ERR_NOSYS     -8  // Niezaimplementowane w danym środowisku

#define TR_NULL ((void*)0)

/* Makra pomocnicze */
#define TR_SUCCESS(s) ((s) >= 0)
#define TR_FAILED(s)  ((s) < 0)

#endif