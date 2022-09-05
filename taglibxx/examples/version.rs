use taglibxx as taglib;

fn main() {
    println!(
        "libtag {}.{}.{}",
        taglib::tag_version_major(),
        taglib::tag_version_minor(),
        taglib::tag_version_patch()
    );
}
