#[cfg(target_os = "windows")]
fn main() {
    porter_build::configure_windows("baker.ico", false)
        .expect("unable to compile Windows resource");

    println!("cargo:rerun-if-changed=build.rs");
}
