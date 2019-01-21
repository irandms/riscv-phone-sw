// build.rs

extern crate cc;

fn main() {
    cc::Build::new()
        .file("src/led_fade.c")
        .include("src")
        .compile("led");
}
