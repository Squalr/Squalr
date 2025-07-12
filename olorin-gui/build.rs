fn main() {
    // Compile user interface .slint files into usable Rust code.
    slint_build::compile("ui/build.slint").unwrap();
}
