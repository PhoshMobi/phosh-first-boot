use std::env;
use std::process::Command;

fn main() {
    glib_build_tools::compile_resources(
        &["resources"],
        "resources/mobi.phosh.FirstBoot.gresource.xml",
        "mobi.phosh.FirstBoot.gresource",
    );

    // Keep Meson build in sync unless PFB_SKIP_MESON=1
    // This is a convenience for local development, distro packaging SHOULD
    // set PFB_SKIP_MESON=1.
    println!("cargo:rerun-if-env-changed=PFB_SKIP_MESON");
    let meson_preconfigured = env::var("PFB_SKIP_MESON").is_ok_and(|value| value == "1");
    if !meson_preconfigured {
        let meson_dir = format!("{}/_build/", env::var("OUT_DIR").unwrap());
        Command::new("meson")
            .args(["setup", "--reconfigure", &meson_dir])
            .status()
            .unwrap();
        Command::new("meson")
            .args(["compile", "-C", &meson_dir])
            .status()
            .unwrap();

        let src = format!("{}/data/gschemas.compiled", &meson_dir);
        // Would be nicer to have it in target/<profile> but .cargo/toml can't have variables
        // in env vars
        let dst = format!(
            "{}/target/gschemas.compiled",
            env::var("CARGO_MANIFEST_DIR").unwrap()
        );
        std::fs::copy(src, dst).unwrap();
    }

    println!("cargo::rerun-if-changed=meson.build");
    println!("cargo::rerun-if-changed=src/meson.build");
    println!("cargo::rerun-if-changed=resources/meson.build");
    println!("cargo::rerun-if-changed=data/meson.build");
    println!("cargo:rustc-link-lib=crypt");
}
