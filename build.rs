// Script to compile `WinSockRawDll` and `WinSockRawDriver` and generate corresponding bindings.
use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// Copy a directory from source to destination recursively.
fn copy_directory(source: impl AsRef<Path>, destination: impl AsRef<Path>) -> io::Result<()> {
    fs::create_dir_all(&destination)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let filetype = entry.file_type()?;
        if filetype.is_dir() {
            copy_directory(entry.path(), destination.as_ref().join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), destination.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}

fn main() {
    let root_path = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    let winsockraw_root_path = PathBuf::from(root_path).join("WinSockRaw");
    let winsockraw_build_root_path = out_path.join("WinSockRaw");

    // Find MSBuild path.
    let program_files_path = PathBuf::from(env::var("ProgramFiles(x86)").unwrap())
                                        .join("Microsoft Visual Studio")
                                        .join("Installer")
                                        .join("vswhere.exe");
    let cmd: Output = Command::new(program_files_path.to_str().unwrap())
                             .args(&["-latest", "-prerelease", "-products", "*", "-requires", "Microsoft.Component.MSBuild", "-find", "MSBuild\\**\\Bin\\MSBuild.exe"])
                             .output().expect("MSBuild path not found!");
    let msbuild_path = std::str::from_utf8(&cmd.stdout).unwrap();

    // Copy the build directory to `OUT_DIR` to avoid packaging issues.
    copy_directory(&winsockraw_root_path, &winsockraw_build_root_path).unwrap();

    // Build `WinSockRawDll` and `WinSockRawDriver`.
    let profile = std::env::var("PROFILE").unwrap();
    let target = std::env::var("TARGET").unwrap();
    let architecture = if target.contains("x86_64") {"x64"} else {"x86"}; 
    let x64 = if architecture == "x64" {architecture} else {""};

    Command::new(msbuild_path.trim_end())
       .args(&[winsockraw_build_root_path.join("WinSockRaw.sln").to_str().unwrap(), 
               "-target:WinSockRawDll;WinSockRawDriver", 
               &format!("-p:Configuration={};Platform={}", profile, architecture)])
       .status().unwrap();
 

    // Link.
    let lib_path: PathBuf = winsockraw_build_root_path.join(&x64).join(&profile);
    println!("cargo:rustc-link-search={}", lib_path.to_str().unwrap());
    println!("cargo:rustc-link-lib=dylib=WinSockRawDll");
    println!("cargo:rerun-if-changed=WinSockRaw\\WinSockRawDll\\winsockraw.h");

    // Generate bindings.
    let bindings = bindgen::Builder::default()
        // The input header we would like to generate bindings for.
        .header(winsockraw_build_root_path.join("WinSockRawDll").join("winsockraw.h").to_str().unwrap())
        // Ignore due to bindgen limitations.
        .opaque_type("_IMAGE_TLS_DIRECTORY64")
        // Tell cargo to invalidate the built crate whenever any of the included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings for winsockraw");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    bindings.write_to_file(out_path.join("bindings.rs"))
            .expect("Couldn't write bindings!");
    
}