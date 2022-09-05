set positional-arguments

# format all (with some special nightly only options that aren't strictly enforced but recommended)
fmt:
    cargo +nightly fmt -- --config group_imports=StdExternalCrate,imports_granularity=Module
    cargo fmt --all

# install
@install *args='':
    cargo install --path ./$1 "$@"

# run
@run *args='':
    cargo run --release --bin "$@"

# test taglib version
taglib:
    cargo run --release --example version

# Print version information
version:
    #!/bin/bash
    cargo build --bin loudgainer --release
    ./target/release/loudgainer --version

# clean source dir
clean:
    #!/bin/bash
    cargo clean
    rm -rf Cargo.lock
    (
        cd mxc
        cargo clean
        rm -rf Cargo.lock
    )
    (
        cd loudgain
        cargo clean
        rm -rf Cargo.lock
    )
    (
        cd taglibxx
        cargo clean
        rm -rf Cargo.lock
    )