#ifndef TRANGORGE_CAPS_H
#define TRANGORGE_CAPS_H

/*
 * caps/caps.h — C ABI złączonego systemu uprawnień TrangorgeOS.
 *
 * Warstwa capabilities (`caps/` w jądrze, implementacja: caps/export.rs)
 * decyduje co danemu worldowi WOLNO, a silnik polityki (`policy/mod.rs`,
 * przenośny port reguł SPARK z policy/ada/policy.adb) — czy przepuszcza
 * daną klasę/operację. Punkt złączenia obu warstw: policy::decide().
 *
 * Symbole po stronie kernela: #[no_mangle] pub extern "C" w caps/export.rs.
 */

/* === syscalls capabilities (dispatch: caps/syscalls.rs) === */
#define SYS_CAP_QUERY    0x1070ull /* a0 = cap_id -> 1/0 dla current world   */
#define SYS_CAP_REQUEST  0x1071ull /* a0 = cap_id (wymaga CAP_ADMIN) -> 0/-1 */
#define SYS_CAP_RELEASE  0x1072ull /* a0 = cap_id -> 0/-1                    */
#define SYS_CAP_AUDIT    0x1073ull /* -> liczba odmów (telemetria)           */

/* === eksport bitmap/odpytywanie (caps/export.rs) === */

/* Bitmapa capabilities current world. */
unsigned int caps_self_bits(void);

/* Bitmapa capabilities danego worlda (0 gdy world nie istnieje). */
unsigned int caps_world_bits(unsigned int world_id);

/* Czy world ma capability o danym ID (1/0). */
int caps_has(unsigned int world_id, unsigned char cap_id);

/* Kopiuje nazwę capability do buf; zwraca długość (-1 gdy brak ID). */
int caps_name(unsigned char cap_id, unsigned char *buf, unsigned int len);

/* Prośba o capability dla targetu (granter = kernel; wymaga ADMIN). */
int caps_request(unsigned int target, unsigned char cap_id);

/* Zwalnia capability danego worlda. */
int caps_release(unsigned int world_id, unsigned char cap_id);

/* Liczba aktywnych worldów. */
unsigned int caps_world_count(void);

/* Liczba zdarzeń audit capabilities. */
unsigned long long caps_audit_count(void);

#endif /* TRANGORGE_CAPS_H */