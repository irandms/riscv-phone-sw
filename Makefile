FIRMWARE      := riscv-phone-sw
# Example firmware (uncomment one)
EXAMPLE       := blinky_clint

# Board crate (uncomment one)
BOARD        := hifive
#BOARD        := lofive

# OpenOCD configuration (uncomment one)
OPENOCD_CFG  := hifive-openocd.cfg
#OPENOCD_CFG  := lofive-openocd.cfg

TARGET       := riscv32imac-unknown-none-elf
TARGET_DIR   := $(abspath ./target/$(TARGET)/debug)
FIRMWARE_DIR := $(TARGET_DIR)
FIRMWARE_BIN := $(FIRMWARE_DIR)/$(FIRMWARE)
EXAMPLE_DIR  := $(TARGET_DIR)/examples
EXAMPLE_BIN  := $(EXAMPLE_DIR)/$(EXAMPLE)

BAUD_RATE := 115200
TTY := /dev/ttyUSB2

build:
	cargo build $(ARGS)

examples:
	cargo build --examples $(ARGS)

test:
	xargo test --all $(ARGS)

clean:
	xargo clean $(ARGS)

readelf:
	llvm-readelf -a -h -s -r -symbols $(FIRMWARE_BIN) $(ARGS)

size:
	llvm-size $(FIRMWARE_BIN) $(ARGS)

objdump:
	llvm-objdump -d -S $(FIRMWARE_BIN) $(ARGS)

dwarfdump:
	llvm-dwarfdump -verify $(FIRMWARE_BIN) $(ARGS) | grep error | wc -l

stcat:
	stty -F $(TTY) $(BAUD_RATE) sane -opost -brkint -icrnl -isig -icanon -iexten -echo
	cat $(TTY) | stcat -e $(FIRMWARE_BIN)

# .gdbinit adds a upload command to gdb
gdb:
	riscv32-unknown-elf-gdb $(FIRMWARE_BIN) $(ARGS)

openocd:
	openocd -f $(OPENOCD_CFG) $(ARGS)

upload:
	openocd -f $(OPENOCD_CFG) \
		-c "flash protect 0 64 last off; program ${FIRMWARE_BIN}; resume 0x20400000; exit"

upload_example:
	openocd -f $(OPENOCD_CFG) \
		-c "flash protect 0 64 last off; program ${EXAMPLE_BIN}; resume 0x20400000; exit"

.PHONY: build test clean readelf size objdump dwarfdump gdb openocd upload
