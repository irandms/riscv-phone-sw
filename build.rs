// build.rs

extern crate cc;

fn main() {
    cc::Build::new()
        .file("examples/chello.c")
        .flag("-march=rv32imac")
        .flag("-mabi=ilp32")
        .flag("-mcmodel=medlow")
        .include("include")
        .compile("chello");
}
