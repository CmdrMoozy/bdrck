use anyhow::{bail, Result};
use std::ffi::OsStr;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::tempdir;

fn runtime_env(key: &str) -> Result<String> {
    Ok(std::env::var(key)?)
}

fn copy_all<F: AsRef<Path>, T: AsRef<Path>>(from: F, to: T) -> Result<()> {
    for e in walkdir::WalkDir::new(from.as_ref())
        .into_iter()
        .filter_entry(|e| e.file_name() != OsStr::new(".git"))
    {
        let entry = e.map_err(|e| {
            if e.io_error().is_some() {
                e.into_io_error().unwrap()
            } else {
                io::Error::new(io::ErrorKind::Other, e.to_string())
            }
        })?;

        let target = to.as_ref().join(entry.path());
        if entry.file_type().is_dir() {
            fs::create_dir_all(&target)?;
        } else {
            if let Err(e) = fs::copy(entry.path(), &target) {
                bail!(
                    "failed to copy {} => {}: {:?}",
                    entry.path().display(),
                    target.display(),
                    e
                );
            }
        }
    }
    Ok(())
}

fn build<S: AsRef<Path>, P: AsRef<Path>>(source: S, prefix: P) -> Result<PathBuf> {
    let source = source.as_ref().join("libsodium");
    let lib = prefix.as_ref().join("lib");

    let configure = match fs::canonicalize(source.join("configure")) {
        Ok(c) => c,
        Err(e) => bail!(
            "canonicalize failed for {}: {:?}",
            source.join("configure").display(),
            e
        ),
    };

    let ret = Command::new(configure)
        .current_dir(&source)
        .arg(format!("--prefix={}", prefix.as_ref().display()))
        .arg(format!("--libdir={}", lib.display()))
        .arg(format!("--host={}", runtime_env("TARGET")?))
        .arg("--enable-shared=no")
        .status()?;
    if !ret.success() {
        bail!("configure failed");
    }

    let ret = Command::new("make")
        .current_dir(&source)
        .env("V", "1")
        .arg("check")
        .arg(format!("-j{}", runtime_env("NUM_JOBS")?))
        .status()?;
    if !ret.success() {
        bail!("make check failed");
    }

    let ret = Command::new("make")
        .current_dir(&source)
        .arg("install")
        .status()?;
    if !ret.success() {
        bail!("make install failed");
    }

    Ok(lib)
}

fn main_impl() -> Result<()> {
    let cargo_out_dir_str = runtime_env("OUT_DIR")?;
    let cargo_out_dir = PathBuf::from(cargo_out_dir_str.clone());

    // libsodium tests fail if it's in a path with spaces:
    // https://github.com/jedisct1/libsodium/issues/207
    // In that case, use a temporary directory instead.
    let (_maybe_tmp, out_dir) = {
        if cargo_out_dir_str.contains(' ') {
            let t = tempdir()?;
            let d = t.path().to_path_buf();
            (Some(t), d)
        } else {
            (None, cargo_out_dir.clone())
        }
    };

    let source_dir = out_dir.join("src");
    let prefix_dir = out_dir.join("prefix");

    copy_all("libsodium", &source_dir)?;

    let lib_dir = build(&source_dir, &prefix_dir)?;
    let include_dir = fs::canonicalize("libsodium/src/libsodium/include")?;

    println!("cargo:rustc-link-lib=static=sodium");
    println!("cargo:rustc-link-search=native={}", lib_dir.display());
    println!("cargo:include={}", include_dir.display());
    println!("cargo:lib={}", lib_dir.display());

    let bindings = bindgen::Builder::default()
        .header(include_dir.join("sodium.h").display().to_string())
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("unable to generate bindings");

    bindings.write_to_file(cargo_out_dir.join("bindings.rs"))?;

    Ok(())
}

fn main() {
    if let Err(e) = main_impl() {
        eprintln!("build failed: {:?}", e);
        panic!("build failed");
    }
}
