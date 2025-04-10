use std::env;

fn main() {
    let target = env::var("TARGET").unwrap();
    println!("cargo:rustc-check-cfg=cfg(armv6m)");
    println!("cargo:rustc-check-cfg=cfg(armv7em)");

    if target.starts_with("thumbv6m-") {
        println!("cargo:rustc-cfg=armv6m");
    } else if target.starts_with("thumbv7em-") {
        println!("cargo:rustc-cfg=armv7em");
    }
}
