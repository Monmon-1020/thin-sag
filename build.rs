fn main() {
    // Foundation をリンク
    println!("cargo:rustc-link-lib=framework=Foundation");
    // NSWorkspace 等を含む AppKit をリンク
    println!("cargo:rustc-link-lib=framework=AppKit");
}
