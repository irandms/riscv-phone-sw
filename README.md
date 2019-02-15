# RISCV-V Phone Software

This repository houses source code and associated files used for building examples and runnable firmware for the RISC-V Phone project, designed by Group 16 in Oregon State University's 2018-2019 Electrical & Computer Engineering Senior Design course. The phone's hardware design is located [here](https://github.com/irandms/riscv-phone). Below are some instructions to help you get started with building and running the code.

Steps to compile the firmware in this project:

1. Install rustc nightly via rustup (recommended method, and nightly is required to build some of the crates used) and set it as the default toolchain.
    * `rustup toolchain install nightly`
    * `rustup default nightly`
2. Install the `riscv32imac-unknown-none-elf` target via rustup
    * `rustup target add riscv32imac-unknown-none-elf`
3. Make sure `xargo` is installed
    * `cargo install xargo`
4. Follow the steps on [riscv-tools](https://github.com/riscv/riscv-tools) to build the 32-bit RISC-V toolchain for things like gcc, ar, ld, etc. If your distribution of choice packages these binaries (`riscv32-unknown-elf-gcc, riscv32-unknown-elf-*`), you may skip this step and possibly step 5.
5. Run `source ./env.sh`, ensuring that the environment variable `$RISCV` is set and that `$RISCV/bin` is in your `$PATH`.
6. Running `make` or `cargo build` should build the entire firmware; running `make upload` will flash a connected HiFive1 or phone board with the compiled binary.

** Note: ** Much of the inital work in this repository is exploratory, and code in the `examples/` directory may not function properly.

# Current Issues:

* The crate `riscv-rt` has a fix for the `panic_implementation` directive that was deprecated in favor of `panic_handler`, but `cargo` will try and check out version 0.3.0 of this crate rather than the most recent commit. Temporary workaround is to apply the latest [commit](https://github.com/rust-embedded/riscv-rt/commit/4204328320fca54f29a90e22bf1f80a54e168109) (commit hash 4204328) to the local copy that cargo checks out; if the firmware doesn't build due to an error that mentions `panic_implementation`, it will show the local path to this file to manually apply this commit/patch.
