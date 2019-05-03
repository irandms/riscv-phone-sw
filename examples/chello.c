#include <stdint.h>
#include <stddef.h>

int puts(const char *s) {
    while (*s != '\0') {
        volatile uint32_t *UART0_TXDATA = (uint32_t *)0x10013000;
        while ((*UART0_TXDATA) & 0x80000000) ;
        (*UART0_TXDATA) = *s;

        if (*s == '\n') {
            while ((*UART0_TXDATA) & 0x80000000) ;
            (*UART0_TXDATA) = '\r';
        }

        ++s;
    }

    return 0;
}

void hello_from_C(void) {
    puts("Hello from C!\n");
}
