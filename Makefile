FIRMWARE      := riscv-phone-sw

# Example firmware (uncomment one), or specify when calling make
#EXAMPLE       := blinky_clint
#EXAMPLE       := nokia5110
#EXAMPLE       := bridge_example
#EXAMPLE       := atomiqueue_example
#EXAMPLE       := max7221

# Board crate (uncomment one)
BOARD        := hifive
#BOARD        := phone-dev-board

# OpenOCD configuration (uncomment one)
OPENOCD_CFG  := hifive-openocd.cfg
#OPENOCD_CFG  := phone-dev-board.cfg

TARGET       := riscv32imac-unknown-none-elf
ifndef RELEASE
TARGET_DIR   := $(abspath ./target/$(TARGET)/debug)
else
TARGET_DIR   := $(abspath ./target/$(TARGET)/release)
endif
FIRMWARE_DIR := $(TARGET_DIR)
FIRMWARE_BIN := $(FIRMWARE_DIR)/$(FIRMWARE)
EXAMPLE_DIR  := $(TARGET_DIR)/examples
EXAMPLE_BIN  := $(EXAMPLE_DIR)/$(EXAMPLE)

RISCV_GDB    := $(abspath $(RISCV_PATH)/bin/riscv64-unknown-elf-gdb)

GDB_UPLOAD_ARGS ?= --batch

GDB_UPLOAD_CMDS += -ex "set remotetimeout 240"
GDB_UPLOAD_CMDS += -ex "target extended-remote localhost:3333"
GDB_UPLOAD_CMDS += -ex "monitor reset halt"
GDB_UPLOAD_CMDS += -ex "monitor flash protect 0 64 last off"
GDB_UPLOAD_CMDS += -ex "load"
GDB_UPLOAD_CMDS += -ex "monitor resume"
GDB_UPLOAD_CMDS += -ex "monitor shutdown"
GDB_UPLOAD_CMDS += -ex "quit"

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
	$(RISCV_GDB) $(FIRMWARE_BIN) $(ARGS)

openocd:
	openocd -f $(OPENOCD_CFG) $(ARGS)

upload:
	openocd -f $(OPENOCD_CFG) & \
	$(RISCV_GDB) -n $(FIRMWARE_BIN) $(GDB_UPLOAD_ARGS) $(GDB_UPLOAD_CMDS) && \
	echo "Successfully uploaded '$(FIRMWARE)' to $(BOARD)."

upload_ex:
	openocd -f $(OPENOCD_CFG) & \
	$(RISCV_GDB) -n $(EXAMPLE_BIN) $(GDB_UPLOAD_ARGS) $(GDB_UPLOAD_CMDS) && \
	echo "Successfully uploaded '$(EXAMPLE_BIN)' to $(BOARD)."

.PHONY: build test clean readelf size objdump dwarfdump gdb openocd upload upload_ex
