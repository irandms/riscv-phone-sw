# riscv-phone-sw

Steps to compile the firmware in this project:

1. Install rustc nightly via rustup and set it as the default toolchain
    * `rustup toolchain install nightly`
    * `rustup default nightly`
2. Install the `riscv32imac-unknown-none-elf` target via rustup
    * `rustup target add riscv32imac-unknown-none-elf`
3. Make sure `xargo` is installed
    * `cargo install xargo`
4. Follow the steps on [riscv-tools](https://github.com/riscv/riscv-tools) to build the 32-bit RISC-V toolchain for things like gcc, ar, ld, etc. If your distribution of choice packages these binaries (`riscv32-unknown-elf-gcc, riscv32-unknown-elf-*`), you may skip this step and possibly step 5.
5. Run `./env.sh`, ensuring that the environment variable `$RISCV` is set and that `$RISCV/bin` is in your `$PATH`.
6. Running `make` should build the entire firmware; running `make upload` will flash a connected HiFive1 or phone board.

Current Issues:

* The crate `riscv-rt` has a fix for the `panic_implementation` directive that was deprecated in favor of `panic_handler`, but `cargo` will try and check out version 0.3.0 of this crate rather than the most recent commit. Temporary workaround is to apply the latest [commit](https://github.com/rust-embedded/riscv-rt/commit/4204328320fca54f29a90e22bf1f80a54e168109) (commit hash 4204328) to the local copy that cargo checks out; if the firmware doesn't build due to an error that mentions `panic_implementation`, it will show the local path to this file to manually apply this commit/patch.
