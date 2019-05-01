set remotetimeout 240
target extended-remote localhost:3333
monitor arm semihosting enable

define upload
  monitor reset halt
  monitor flash protect 0 64 last off
  load
  monitor flash protect 0 64 last on
  continue
end
document upload
Upload program to hifive board
end

# Load Rust's GDB pretty printers
directory ~/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/etc/
add-auto-load-safe-path ~/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/etc/
