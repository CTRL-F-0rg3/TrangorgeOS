/*
 * test.c — smoke test proving the C bindings enforce the same rule as Rust:
 * only authorized requests are forwarded.
 */
#include "tgcomm.h"
#include <stdio.h>
#include <string.h>

static int failures = 0;

#define CHECK(cond)                                                        \
    do {                                                                   \
        if (!(cond)) {                                                     \
            fprintf(stderr, "FAIL %s:%d: %s\n", __FILE__, __LINE__, #cond);\
            failures++;                                                    \
        }                                                                  \
    } while (0)

int main(void) {
    /* Layout sanity. */
    CHECK(sizeof(tgcomm_msg_t) == 64);
    CHECK(tgcomm_msg_size() == 64);
    CHECK(tgcomm_version() == TGCOMM_VERSION);
    CHECK(tgcomm_cap_table_size() <= sizeof(tgcomm_cap_table_t));

    /* Build a driver->kernel video request. */
    tgcomm_msg_t m;
    tgcomm_build_request(&m, TGCOMM_LAYER_DRIVERSPACE, TGCOMM_LAYER_KERNEL,
                         TGCOMM_OPCODE(TGCOMM_OP_VIDEO, 1), /*cap=*/7u);
    CHECK(tgcomm_msg_valid(&m) != 0);

    /* A capability table is opaque to C; we only reserve its storage. */
    tgcomm_cap_table_t table;
    tgcomm_cap_table_init(&table);

    /* Without capability 7, the request must be denied. */
    CHECK(tgcomm_authorize(&table, &m) != TGCOMM_FORWARDED);

    /* Grant capability 7: CALL over a Device. */
    CHECK(tgcomm_cap_insert(&table, 7u, 42u, TGCOMM_OBJ_DEVICE,
                            TGCOMM_RIGHT_CALL) == 0);
    CHECK(tgcomm_authorize(&table, &m) == TGCOMM_FORWARDED);

    /* Granting READ instead of CALL must NOT authorize a CALL. */
    tgcomm_cap_table_t table2;
    tgcomm_cap_table_init(&table2);
    tgcomm_cap_insert(&table2, 7u, 42u, TGCOMM_OBJ_DEVICE, TGCOMM_RIGHT_READ);
    CHECK(tgcomm_authorize(&table2, &m) == TGCOMM_RIGHTS_INSUFFICIENT);

    /* A forbidden route (userspace -> driverspace) must be denied. */
    tgcomm_msg_t bad = m;
    bad.layer = TGCOMM_LAYER_USERSPACE;
    bad.target = TGCOMM_LAYER_DRIVERSPACE;
    CHECK(tgcomm_authorize(&table, &bad) == TGCOMM_ROUTE_DENIED);

    /* Revoking the capability flips the request back to a deny. */
    CHECK(tgcomm_cap_remove(&table, 7u) != 0);
    CHECK(tgcomm_authorize(&table, &m) == TGCOMM_NO_CAPABILITY);

    /* Ring round-trip. */
    {
        uint8_t ring[TGCOMM_RING_CTRL + 4u * TGCOMM_MSG_SIZE];
        tgcomm_ring_init(ring, 4u);
        tgcomm_msg_t out;
        memset(&out, 0, sizeof(out));
        CHECK(tgcomm_ring_push(ring, &m) == 0);
        CHECK(tgcomm_ring_available(ring) == 1u);
        CHECK(tgcomm_ring_pop(ring, &out) == 0);
        CHECK(memcmp(&m, &out, sizeof(m)) == 0);
        CHECK(tgcomm_ring_available(ring) == 0u);
    }

    if (failures == 0) {
        printf("lib-C smoke test: ALL CHECKS PASSED\n");
        return 0;
    }
    fprintf(stderr, "lib-C smoke test: %d FAILURES\n", failures);
    return 1;
}
