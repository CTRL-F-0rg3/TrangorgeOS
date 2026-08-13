#include <stdint.h>
#include <string.h>

static uint8_t buffer[16];

int main(void)
{
    uint64_t r0 = 0;
    uint64_t r1 = 0;
    uint64_t r2 = 0;
fn_main:
    r0 = 16ull;
    r1 = 32ull;
    r2 = r0 + r1;
    memset(buffer, (int)0ull, 16);
    buffer[0] = (uint8_t)65ull;
    buffer[1] = (uint8_t)66ull;
    buffer[2] = (uint8_t)67ull;
    r0 = buffer[0];
    r1 = buffer[1];
    r2 = r0 + r1;
    if (r2 != 131ull) goto else_1;
    r0 = 1ull;
    goto end_2;
else_1:
    r0 = 0ull;
end_2:
loop_3:
    if (r0 == 0ull) goto loop_end_4;
    r1 = r0 - 1ull;
    r0 = r1;
    goto loop_3;
loop_end_4:
    return (int)r0;
}
