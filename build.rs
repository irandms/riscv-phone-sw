// build.rs

extern crate cc;

fn main() {
    cc::Build::new()
        .file("examples/chello.c")
        .include("include")
        .compile("chello");
}
