#!/bin/sh

export XARGO_RUST_SRC=~/git/rust/src
export LD_LIBRARY_PATH=$RISCV/lib

# Used by build.rs and cc crate
export TARGET_CC=$RISCV/bin/riscv32-unknown-elf-gcc
#export CFLAGS="-fno-builtin-printf -g -march=rv32imac -mabi=ilp32 -mcmodel=medany"
export AR=$RISCV/bin/riscv32-unknown-elf-gcc-ar
export CC_riscv32imac_unknown_none_elf=$RISCV/bin/riscv32-unknown-elf-gcc
