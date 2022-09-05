use std::path::PathBuf;

#[cfg(feature = "build")]
mod build {
    use std::path::PathBuf;
    use std::process::Command;
    use std::{env, fs, io, str};

    use cmake::Config;
    #[cfg(feature = "taglib112")]
    fn build_folder() -> &'static str {
        "taglib-1.12"
    }

    #[cfg(not(feature = "taglib112"))]
    fn build_folder() -> &'static str {
        "taglib-master"
    }

    fn output() -> PathBuf {
        PathBuf::from(env::var("OUT_DIR").unwrap())
    }

    fn source() -> PathBuf {
        output().join(build_folder())
    }

    fn fetch() -> io::Result<()> {
        let output_base_path = output();
        let clone_dest_dir = build_folder();
        let _ = std::fs::remove_dir_all(output_base_path.join(&clone_dest_dir));
        let status = if cfg!(feature = "taglib112") {
            Command::new("git")
                .current_dir(&output_base_path)
                .arg("clone")
                .arg("--depth=1")
                .arg("-b")
                .arg("v1.12")
                .arg("--single-branch")
                .arg("https://github.com/taglib/taglib")
                .arg(&clone_dest_dir)
                .status()?
        } else {
            Command::new("git")
                .current_dir(&output_base_path)
                .arg("clone")
                .arg("--depth=1")
                .arg("https://github.com/taglib/taglib")
                .arg(&clone_dest_dir)
                .status()?
        };

        if status.success() {
            Ok(())
        } else {
            Err(io::Error::new(io::ErrorKind::Other, "fetch failed"))
        }
    }

    pub fn build() -> (PathBuf, PathBuf) {
        if fs::metadata(&source()).is_err() {
            fs::create_dir_all(&output()).expect("failed to create build directory");
            fetch().unwrap();
        }
        let dst = Config::new(source())
            .define("BUILD_SHARED_LIBS", "OFF")
            .define("ENABLE_STATIC_RUNTIME", "ON")
            .define("CMAKE_BUILD_TYPE", "Release")
            .build();
        (dst.join("include").join("taglib"), dst.join("lib"))
    }
}

#[cfg(not(feature = "build"))]
#[cfg(not(target_env = "msvc"))]
fn try_vcpkg(_statik: bool) -> Option<(PathBuf, PathBuf)> {
    None
}

#[cfg(not(feature = "build"))]
#[cfg(target_env = "msvc")]
fn try_vcpkg(statik: bool) -> Option<(PathBuf, PathBuf)> {
    if !statik {
        env::set_var("VCPKGRS_DYNAMIC", "1");
    }

    vcpkg::find_package("taglib")
        .map_err(|e| {
            println!("Could not find taglib with vcpkg: {}", e);
        })
        .map(|l| (l.include_paths[0], l.link_paths[0]))
        .ok()
}

/// returns (include_path, lib_path)
pub fn get_taglib_paths() -> (PathBuf, PathBuf) {
    #[cfg(feature = "build")]
    return build::build();

    #[cfg(not(feature = "build"))]
    if let Some(paths) = try_vcpkg(cfg!(feature = "static")) {
        paths
    }
    // Fallback to pkg-config
    else {
        let pkg = pkg_config::Config::new()
            .statik(cfg!(feature = "static"))
            .probe("taglib")
            .unwrap();
        (pkg.include_paths[0].clone(), pkg.link_paths[0].clone())
    }
}
