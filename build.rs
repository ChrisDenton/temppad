fn main() {
    println!("cargo::rustc-link-arg-bins=/entry:main");
    println!("cargo::rustc-link-arg-bins=/defaultlib:libvcruntime.lib");
    // A manifest file is necessary so it doesn't look awful on high DPI displays
    println!("cargo::rustc-link-arg-bins=/MANIFEST:EMBED");
    println!("cargo::rustc-link-arg-bins=/MANIFESTINPUT:./manifest.xml");
}
