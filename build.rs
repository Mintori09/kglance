fn main() {
    slint_build::compile("src/ui/window.slint")
        .expect("Slint compilation failed");
}
