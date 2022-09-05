use std::fs::{File, OpenOptions};
use std::io::prelude::*;
use std::path::PathBuf;
use std::process::Command;
use std::{env, fs, io};

use cmake::Config;

fn output() -> PathBuf {
    PathBuf::from(env::var("OUT_DIR").unwrap())
}

fn source() -> PathBuf {
    output().join("loudgain-master")
}

fn ebur_source() -> PathBuf {
    output().join("ebur__016")
}

fn cebur() -> PathBuf {
    output().join("cebur")
}

fn fetch() -> io::Result<()> {
    let source = source();
    let _ = std::fs::remove_dir_all(&source);
    let status = if env::var("CARGO_FEATURE_FLOAT").is_ok() {
        Command::new("git")
            .current_dir(&output())
            .arg("clone")
            .arg("--depth=1")
            .arg("-b")
            .arg("f64")
            //my fork has TagLib version print enabled
            .arg("https://github.com/sagudev/loudgain")
            .arg(&source)
            .status()
    } else {
        Command::new("git")
            .current_dir(&output())
            .arg("clone")
            .arg("--depth=1")
            //.arg("-b")
            //.arg("libebur_search")
            //my fork has TagLib version print enabled
            .arg("https://github.com/sagudev/loudgain")
            .arg(&source)
            .status()
    }?;

    // id on linux patch
    if cfg!(unix) && env::var("CARGO_FEATURE_EBUR128").is_ok() {
        let path = source.join("cmake").join("FindEBUR128.cmake");
        let mut f = File::open(&path)?;
        let mut content = "SET(CMAKE_FIND_LIBRARY_SUFFIXES .a)".as_bytes().to_owned();
        f.read_to_end(&mut content)?;

        let mut f = File::create(&path)?;
        f.write_all(content.as_slice())?;
    }

    if status.success() {
        Ok(())
    } else {
        Err(io::Error::new(io::ErrorKind::Other, "fetch failed"))
    }
}

fn build_ebur() -> io::Result<()> {
    let source = ebur_source();
    let _ = std::fs::remove_dir_all(&source);
    let status = Command::new("git")
        .current_dir(&output())
        .arg("clone")
        .arg("--depth=1")
        .arg("-b")
        .arg("0.1.6")
        .arg("--single-branch")
        //my fork has TagLib version print enabled
        .arg("https://github.com/sdroege/ebur128")
        .arg(&source)
        .status()?;

    if !status.success() {
        return Err(io::Error::new(io::ErrorKind::Other, "fetch failed"));
    }

    //[workspace]
    //members = [".", "crate-bin"]
    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .open(source.join("Cargo.toml"))
        .unwrap();

    writeln!(file, "[workspace]")?;

    let cebur = cebur();
    let status = Command::new("cargo")
        .current_dir(&source)
        .arg("cinstall")
        .arg("--release")
        .arg("--target-dir")
        .arg(&cebur)
        .arg("--prefix")
        .arg(&cebur)
        //.arg("--library-type")
        //.arg("staticlib")
        .status()?;

    if !status.success() {
        return Err(io::Error::new(io::ErrorKind::Other, "cinstall failed"));
    }

    Ok(())
}

fn main() {
    let (_, taglib_lib) = taglib::get_taglib_paths();
    if fs::metadata(&source()).is_err() {
        fs::create_dir_all(&output()).expect("failed to create build directory");
        fetch().unwrap();
    }
    let tlib = taglib_lib.join("pkgconfig").display().to_string();
    let pcp = match env::var("PKG_CONFIG_PATH") {
        Ok(val) => format!("{}:{}", tlib, val),
        Err(_) => tlib,
    };

    // use rust ebur128 c-api
    if env::var("CARGO_FEATURE_EBUR128").is_ok() {
        build_ebur().unwrap();
        //pcp += ":";
        //pcp += &cebur().join("lib").join("pkgconfig").display().to_string();
        println!("Ebur128 built in {}", cebur().display());
    }

    let mut dst = Config::new(source());

    dst.define("CMAKE_BUILD_TYPE", "Release")
        .define(
            "CMAKE_EXE_LINKER_FLAGS",
            format!("-L {}", taglib_lib.display()),
        )
        .env("PKG_CONFIG_PATH", pcp);

    if env::var("CARGO_FEATURE_EBUR128").is_ok() {
        dst.define("CMAKE_PREFIX_PATH", cebur().display().to_string());
    };
    let dst = dst.build().join("bin");
    let base = directories::BaseDirs::new().unwrap();
    let dir = base.data_dir();
    let e = fs::read_dir(dst)
        .unwrap()
        .map(|res| res.unwrap())
        .find(|e| e.file_name().to_string_lossy().contains("loudgain"))
        .unwrap();
    let target = dir.join(e.file_name().to_string_lossy().to_string());
    fs::copy(e.path(), &target).unwrap();
    fs::write("src/exec.txt", target.display().to_string()).expect("Unable to write file");
}
