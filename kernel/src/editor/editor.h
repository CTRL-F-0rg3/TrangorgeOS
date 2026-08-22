#ifndef KERNEL_EDITOR_H
#define KERNEL_EDITOR_H

#define ED_MAX_LINES 512
#define ED_LINE_LEN  128

/* kody rozszerzone z k_input_keycode() */
#define EDK_ENTER      0x100
#define EDK_BACKSPACE  0x101
#define EDK_ESC        0x102
#define EDK_RIGHT      0x103
#define EDK_LEFT       0x104
#define EDK_DOWN       0x105
#define EDK_UP         0x106
#define EDK_HOME       0x107
#define EDK_END        0x108
#define EDK_DELETE     0x109
#define EDK_TAB        0x10A
#define EDK_F5         0x110
#define EDK_F8         0x111

int editor_run(const char *path);

#endif