fn main() {
    let ico = "forge.ico";
    porter_build::configure_windows(ico, false).expect("unable to compile Windows resource");
    porter_build::configure_linux(ico).expect("unable to compile Mac resource");
    porter_build::configure_macos(ico).expect("unable to compile Linux resource");

    println!("cargo:rerun-if-changed=build.rs");
}
