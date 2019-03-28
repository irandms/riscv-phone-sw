#!/bin/sh

export XARGO_RUST_SRC=~/git/rust/src
export LD_LIBRARY_PATH=$RISCV_PATH/lib

# Used by build.rs and cc crate
#export CFLAG_DBG="-ffunction-sections -fdata-sections -g -O0"
export CFLAGS="-fno-builtin-printf -march=rv32imac -mabi=ilp32 -mcmodel=medlow"
