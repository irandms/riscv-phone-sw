# RISCV-V Phone Software

This repository houses source code and associated files used for building examples and runnable firmware for the RISC-V Phone project, designed by Group 16 in Oregon State University's 2018-2019 Electrical & Computer Engineering Senior Design course. The phone's hardware design is located [here](https://github.com/irandms/riscv-phone). Below are some instructions to help you get started with building and running the code.

Steps to compile the firmware in this project:

1. Install rustc nightly via [rustup](https://rustup.rs/) (recommended method, and nightly is required to build) and set it as the default toolchain. (
    * `rustup toolchain install nightly`
    * `rustup default nightly`
2. Install the `riscv32imac-unknown-none-elf` target via rustup
    * `rustup target add riscv32imac-unknown-none-elf`
3. Install the RISC-V GNU Embedded Toolchain and OpenOCD. SiFive provides system binaries [here](https://www.sifive.com/boards) with install instructions [here](https://github.com/sifive/freedom-e-sdk), although the most pertinent steps are as follows:
    * Download the appropriate .tar.gz for your platform (both the GNU Toolchain and OpenOCD)
    * Unpack each to its own desired location/folder
    * Create the RISCV_OPENOCD_PATH and RISCV_PATH environment variables in your shell of choice, adding both bin folders to your PATH. For now you will also need CC_riscv32imac_unknown_none_elf (to satisfy the cargo setup). Below is an example of the necessary environment variables:
```
export RISCV_OPENOCD_PATH=/my/desired/location/openocd
export RISCV_PATH=/my/desired/location/riscv64-unknown-elf-gcc-<date>-<version>
export PATH=$PATH:$RISCV_PATH/bin:$RISCV_OPENOCD_PATH/bin
export CC_riscv32imac_unknown_none_elf=riscv64-unknown-elf-gcc
```
4. Running `make` or `cargo build` should build the entire firmware; running `make upload` will flash a connected HiFive1 or phone board with the compiled binary. Note that to compile and run examples, you must do `cargo build --examples` and `make upload EXAMPLE=<examplename>` instead. To compile and run release versions, do `cargo build --examples --release` and `make upload EXAMPLE=<examplename> RELEASE=true`

** Note: ** Much of the inital work in this repository is exploratory, and code in the `examples/` directory may not function properly.

# Current Issues:
