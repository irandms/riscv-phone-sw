#include <stdio.h>          // printf over UART!! :)
#include <stdint.h>         // well-defined integer types
#include <stdbool.h>
#include <stdlib.h>
#include "platform.h"       // Contains macros for addresses of SPI registers
#include "max7221.h"

// TX and RX registers aren't defined in platform.h, which includes devices/spi.h
// The FE310-G000 datasheet lists these registers as `txdata` and `rxdata`,
// but devices/spi.h uses `txfifo` and `rxfifo`.

void SPI1_init(uint32_t sck_freq) {
    uint8_t sck_div = get_cpu_freq() / 2 * sck_freq - 1;
    SPI1_REG(SPI_REG_SCKDIV) = sck_div;
    GPIO_REG(GPIO_IOF_SEL) &= ~IOF0_SPI1_MASK;
    GPIO_REG(GPIO_IOF_EN) |= IOF0_SPI1_MASK;
    printf("%d", sck_div);
}

uint8_t SPI1_xfer(uint8_t byte_to_send) {
    /*
    while(SPI1_REG(SPI_REG_TXFIFO) & SPI_TXFIFO_FULL);
    GPIO_REG(GPIO_OUTPUT_VAL) &= ~mask;
    SPI1_REG(SPI_REG_TXFIFO) = byte_to_send;

    volatile uint32_t read = SPI1_REG(SPI_REG_RXFIFO);
    while(read & SPI_RXFIFO_EMPTY) {
        read = SPI1_REG(SPI_REG_RXFIFO);
    };

    if(end_cs) {
        GPIO_REG(GPIO_OUTPUT_VAL) |= mask;
    }

    return read & 0xFF;
    */
	volatile int32_t x;
	while (SPI1_REG(SPI_REG_TXFIFO) & SPI_TXFIFO_FULL);
	SPI1_REG(SPI_REG_TXFIFO) = byte_to_send;

	while ((x = SPI1_REG(SPI_REG_RXFIFO)) & SPI_RXFIFO_EMPTY);

	return x & 0xFF;
}

uint16_t SPI1_xfer16(uint8_t b1, uint8_t b2) {
    uint16_t result = 0;

    SPI1_REG(SPI_REG_CSMODE) = 2;
    result = SPI1_xfer(b1);
    result <<= 8;

    SPI1_REG(SPI_REG_CSMODE) = 0;
    result |= SPI1_xfer(b2);

    return result;
}

int SPI1_write_reg(int register_addr, int value) {
    int reg_read = SPI1_REG(register_addr);
    printf("register 0x%-8x     0x%-8x, %d\n", register_addr, reg_read, reg_read);
    printf("writing value %x into register %-8x\n", value, register_addr);
    SPI1_REG(register_addr) = value;
    reg_read = SPI1_REG(register_addr);
    printf("register 0x%-8x     0x%-8x, %d\n", register_addr, reg_read, reg_read);
    return reg_read;
}

int SPI1_read_reg(int register_addr) {
    int reg_read = SPI1_REG(register_addr);
    printf("register 0x%-8x     0x%-8x, %d\n", register_addr, reg_read, reg_read);
    return reg_read;
}

void MAX7221_init() {
    // Init sequence for MAX7221 (from datasheet)
    SPI1_REG(SPI_REG_CSMODE) = 2;
    SPI1_xfer(0x09); // Decode mode register
    SPI1_xfer(0xFF); // Code B decoder for all digits
    SPI1_REG(SPI_REG_CSMODE) = 3;

    SPI1_REG(SPI_REG_CSMODE) = 2;
    SPI1_xfer(0x0A); // Brightness/intensity register
    SPI1_xfer(0x0F); // MAXIMUM BRIGHTNESS
    SPI1_REG(SPI_REG_CSMODE) = 3;

    SPI1_REG(SPI_REG_CSMODE) = 2;
    SPI1_xfer(0x0B); // Scan-limit control register
    SPI1_xfer(0x03); // Display/control only four digits
    SPI1_REG(SPI_REG_CSMODE) = 3;

    SPI1_REG(SPI_REG_CSMODE) = 2;
    SPI1_xfer(0x0C); // Normal/Shutdown register
    SPI1_xfer(0x01); // Normal mode, not shutdown
    SPI1_REG(SPI_REG_CSMODE) = 3;

    SPI1_REG(SPI_REG_CSMODE) = 2;
    SPI1_xfer(0x0F); // Normal/Test Mode register
    SPI1_xfer(0x00); // Disable test mode
    SPI1_REG(SPI_REG_CSMODE) = 3;
}

uint8_t display_value(uint16_t val) {
    uint8_t dispstr[32] = { '\0' };
    uint8_t deststr[32] = { '\0' };
    uint16_t valcopy = val;
    // Display 4-digit value
    for(int8_t k = 4; k > 0; k--) {
        uint8_t digval = val % 10;
        SPI1_REG(SPI_REG_CSMODE) = 2;
        SPI1_xfer(k); // Which digit
        dispstr[k-1] = SPI1_xfer(digval); // Value of input
        SPI1_REG(SPI_REG_CSMODE) = 3;
        val /= 10;
    }
    deststr[0] = dispstr[3] + '0';
    deststr[1] = dispstr[0] + '0';
    deststr[2] = dispstr[1] + '0';
    deststr[3] = dispstr[2] + '0';

    if(atoi(deststr) != valcopy && (atoi(deststr) + 1000 != valcopy) && (atoi(deststr) - 9000 != valcopy)) {
        printf("MISMATCH!\n");
        printf("%d, %d\n", atoi(deststr), valcopy);
    }
}

void custom_delay(uint16_t v) {
    for(volatile int i = 0; i < 100000; i++) {
        //printf("v: %d\n", v);
    }
}
