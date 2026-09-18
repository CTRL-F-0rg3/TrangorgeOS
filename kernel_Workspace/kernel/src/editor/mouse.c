#include "mouse.h"

static inline void outb(uint16_t port, uint8_t val)
{
    __asm__ volatile("outb %0, %1" :: "a"(val), "Nd"(port));
}

static inline uint8_t inb(uint16_t port)
{
    uint8_t v;
    __asm__ volatile("inb %1, %0" : "=a"(v) : "Nd"(port));
    return v;
}

static void wait_write(void)
{
    while (inb(0x64) & 0x02) { }
}

static void wait_read(void)
{
    while (!(inb(0x64) & 0x01)) { }
}

static void mouse_cmd(uint8_t cmd)
{
    wait_write(); outb(0x64, 0xD4);
    wait_write(); outb(0x60, cmd);
    wait_read();  inb(0x60);
}

static void mouse_rate(uint8_t r)
{
    wait_write(); outb(0x64, 0xD4);
    wait_write(); outb(0x60, 0xF3);
    wait_read();  inb(0x60);

    wait_write(); outb(0x64, 0xD4);
    wait_write(); outb(0x60, r);
    wait_read();  inb(0x60);
}

void mouse_init(void)
{
    wait_write(); outb(0x64, 0xA8);   

    mouse_cmd(0xF6);                 

    mouse_rate(200);
    mouse_rate(100);
    mouse_rate(80);

    mouse_cmd(0xF4);                  
}

int mouse_poll(int *dx, int *dy, int *dz, int *buttons)
{
    static uint8_t pkt[4];
    static int idx = 0;
    int got = 0;

    while (inb(0x64) & 0x01) {
        uint8_t st = inb(0x64);
        uint8_t d = inb(0x60);

        if (!(st & 0x20)) {
            continue;                 
        }

        if (idx == 0 && !(d & 0x08)) {
            continue;                
        }

        pkt[idx++] = d;

        if (idx == 4) {
            idx = 0;

            *buttons = pkt[0] & 0x07;

            *dx = (int)pkt[1] - (int)((pkt[0] << 4) & 0x100);
            *dy = (int)pkt[2] - (int)((pkt[0] << 3) & 0x100);
            *dz = (int)(int8_t)pkt[3];

            got = 1;
        }
    }

    return got;
}