# Example firmware (uncomment one), or specify when calling make
#EXAMPLE       := blinky_clint
#EXAMPLE       := nokia5110
#EXAMPLE       := atomiqueue_example

# OpenOCD configuration (uncomment one)
OPENOCD_CFG  := hifive-openocd.cfg

TARGET       := riscv32imac-unknown-none-elf

ifndef RELEASE
TARGET_DIR   := $(abspath ./target/$(TARGET)/debug)
else
TARGET_DIR   := $(abspath ./target/$(TARGET)/release)
endif

ifdef EXAMPLE
FIRMWARE     := $(TARGET_DIR)/examples/$(EXAMPLE)
else
FIRMWARE     := $(TARGET_DIR)/$(shell basename $(PWD))
endif

RISCV_GDB    := $(abspath $(RISCV_PATH)/bin/riscv64-unknown-elf-gdb)

TTY := /dev/ttyUSB1
BAUD_RATE := 115200

build:
	cargo build $(ARGS)

examples:
	cargo build --examples $(ARGS)

test:
	xargo test --all $(ARGS)

clean:
	xargo clean $(ARGS)

readelf:
	llvm-readelf -a -h -s -r -symbols $(FIRMWARE) $(ARGS)

size:
	llvm-size $(FIRMWARE) $(ARGS)

objdump:
	llvm-objdump -d -S $(FIRMWARE) $(ARGS)

dwarfdump:
	llvm-dwarfdump -verify $(FIRMWARE) $(ARGS) | grep error | wc -l

stcat:
	stty -F $(TTY) $(BAUD_RATE) sane -opost -brkint -icrnl -isig -icanon -iexten -echo
	cat $(TTY) | stcat -e $(FIRMWARE)

# .gdbinit adds a upload command to gdb
gdb:
	$(RISCV_GDB) $(FIRMWARE) $(ARGS)

openocd:
	openocd -f $(OPENOCD_CFG) $(ARGS)

upload:
	openocd -f $(OPENOCD_CFG) & \
	$(RISCV_GDB) $(FIRMWARE) -x upload.gdb && \
	echo "Successfully uploaded '$(FIRMWARE)' to $(BOARD)."

debug: $(FIRMWARE)
	openocd -f $(OPENOCD_CFG) & \
	$(RISCV_GDB) $(FIRMWARE) -x debug.gdb

reset:
	openocd -f $(OPENOCD_CFG) & \
	$(RISCV_GDB) -x reset.gdb

.PHONY: build test clean readelf size objdump dwarfdump gdb openocd upload upload
