fn main() {
    let ico = "baker.ico";
    porter_build::configure_windows(ico, false).expect("unable to compile Windows resource");
    porter_build::configure_linux(ico).expect("unable to compile Windows resource");
    porter_build::configure_macos(ico).expect("unable to compile Windows resource");

    println!("cargo:rerun-if-changed=build.rs");
}
