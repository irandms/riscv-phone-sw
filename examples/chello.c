#include <stdint.h>
#include "../include/platform.h"

int puts(const char *s) {
    while (*s != '\0') {
        while (UART0_REG(UART_REG_TXFIFO) & 0x80000000) ;
        UART0_REG(UART_REG_TXFIFO) = *s;

        if (*s == '\n') {
            while (UART0_REG(UART_REG_TXFIFO) & 0x80000000) ;
            UART0_REG(UART_REG_TXFIFO) = '\r';
        }

        ++s;
    }

    return 0;
}

void hello_from_C(void) {
    puts("Hello from C!\n");

    uint16_t r=0xFF;
    uint16_t g=0;
    uint16_t b=0;

    // Set up RGB PWM

    PWM1_REG(PWM_CFG)   = 0;
    // To balance the power consumption, make one left, one right, and one center aligned.
    PWM1_REG(PWM_CFG)   = (PWM_CFG_ENALWAYS) | (PWM_CFG_CMP2CENTER);
    PWM1_REG(PWM_COUNT) = 0;

    // Period is approximately 244 Hz
    // the LEDs are intentionally left somewhat dim, 
    // as the full brightness can be painful to look at.
    PWM1_REG(PWM_CMP0)  = 0;

    GPIO_REG(GPIO_IOF_SEL)    |= ( (1 << GREEN_LED_OFFSET) | (1 << BLUE_LED_OFFSET) | (1 << RED_LED_OFFSET));
    GPIO_REG(GPIO_IOF_EN )    |= ( (1 << GREEN_LED_OFFSET) | (1 << BLUE_LED_OFFSET) | (1 << RED_LED_OFFSET));
    GPIO_REG(GPIO_OUTPUT_XOR) &= ~( (1 << GREEN_LED_OFFSET) | (1 << BLUE_LED_OFFSET));
    GPIO_REG(GPIO_OUTPUT_XOR) |= (1 << RED_LED_OFFSET);

    while(1){
        volatile uint64_t *now = (volatile uint64_t*)(CLINT_CTRL_ADDR + CLINT_MTIME);
        volatile uint64_t then = *now + 100;
        while (*now < then) { }

        if(r > 0 && b == 0){
            r--;
            g++;
        }
        if(g > 0 && r == 0){
            g--;
            b++;
        }
        if(b > 0 && g == 0){
            r++;
            b--;
        }

        uint32_t G = g;
        uint32_t R = r;
        uint32_t B = b;

        PWM1_REG(PWM_CMP1)  = G << 4;            // PWM is low on the left, GPIO is low on the left side, LED is ON on the left.
        PWM1_REG(PWM_CMP2)  = (B << 1) << 4;     // PWM is high on the middle, GPIO is low in the middle, LED is ON in the middle.
        PWM1_REG(PWM_CMP3)  = 0xFFFF - (R << 4); // PWM is low on the left, GPIO is low on the right, LED is on on the right.
    }
}
