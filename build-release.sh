#!/bin/sh

set -eu

cross_bin="${CROSS:-cross}"
zigbuild_bin="${CARGO_ZIGBUILD:-cargo-zigbuild}"
out_dir="${OUT_DIR:-.}"
target_root="${RELEASE_TARGET_ROOT:-target/release-build}"
build_std_toolchain="${ZIGBUILD_BUILD_STD_TOOLCHAIN:-nightly}"
project_dir="$(pwd -P)"
tmp="${TMPDIR:-/tmp}/ja-colloquial-release.$$"

default_cross_targets='
x86_64-unknown-linux-musl
aarch64-unknown-linux-musl
x86_64-unknown-freebsd
x86_64-unknown-netbsd
x86_64-unknown-illumos
'

default_zigbuild_targets='
x86_64-apple-darwin
aarch64-apple-darwin
riscv64gc-unknown-linux-musl
powerpc64le-unknown-linux-musl
s390x-unknown-linux-musl
'

cross_targets="${CROSS_TARGETS-$default_cross_targets}"
zigbuild_targets="${ZIGBUILD_TARGETS-$default_zigbuild_targets}"

cleanup() {
    rm -rf "$tmp"
}
trap cleanup EXIT HUP INT TERM

mkdir -p "$out_dir" "$tmp"
out_dir="$(cd "$out_dir" && pwd -P)"
case "$target_root" in
    /*) ;;
    *) target_root="${project_dir}/${target_root}" ;;
esac

need_command() {
    command_name="$1"
    if ! command -v "$command_name" >/dev/null 2>&1; then
        printf 'missing required command: %s\n' "$command_name" >&2
        exit 1
    fi
}

need_command cargo
need_command rustup
need_command "$cross_bin"
need_command "$zigbuild_bin"
need_command zig

RUSTDOCFLAGS='-D warnings' cargo doc \
    --locked \
    --no-deps \
    --target-dir "$tmp/docs-target"

is_musl_target() {
    case "$1" in
        *-linux-musl) return 0 ;;
        *) return 1 ;;
    esac
}

is_darwin_target() {
    case "$1" in
        *-apple-darwin) return 0 ;;
        *) return 1 ;;
    esac
}

macos_sdkroot() {
    case "${SDKROOT:-}" in
        '') return 1 ;;
        /*) sdkroot_path="$SDKROOT" ;;
        *) sdkroot_path="${project_dir}/${SDKROOT}" ;;
    esac
    if [ ! -d "$sdkroot_path" ]; then
        printf 'SDKROOT does not exist: %s\n' "$sdkroot_path" >&2
        exit 1
    fi
    (cd "$sdkroot_path" && pwd -P)
}

requires_build_std() {
    case "$1" in
        s390x-unknown-linux-musl) return 0 ;;
        *) return 1 ;;
    esac
}

dynamic_suffix() {
    case "$1" in
        *-apple-darwin) printf '%s\n' dylib ;;
        *) printf '%s\n' so ;;
    esac
}

prepare_source() {
    target="$1"
    source_dir="${tmp}/source-${target}"
    mkdir -p "$source_dir" "$source_dir/.cargo"
    cp -R "$project_dir/src" "$source_dir/src"
    cp -R "$project_dir/include" "$source_dir/include"
    cp "$project_dir/build.rs" "$source_dir/build.rs"
    cp "$project_dir/Cargo.lock" "$source_dir/Cargo.lock"
    cp "$project_dir/README.md" "$source_dir/README.md"
    cp "$project_dir/CHANGELOG.md" "$source_dir/CHANGELOG.md"
    cp "$project_dir/LICENSE" "$source_dir/LICENSE"

    if is_musl_target "$target"; then
        sed 's/^crate-type = .*/crate-type = ["staticlib"]/' \
            "$project_dir/Cargo.toml" >"$source_dir/Cargo.toml"
    else
        sed 's/^crate-type = .*/crate-type = ["staticlib", "cdylib"]/' \
            "$project_dir/Cargo.toml" >"$source_dir/Cargo.toml"
    fi

    printf '%s\n' \
        '[build]' \
        'rustflags = ["--cfg", "ja_colloquial_c_artifact"]' \
        >"$source_dir/.cargo/config.toml"

    if is_darwin_target "$target" && [ -n "${SDKROOT:-}" ]; then
        sdkroot_path="$(macos_sdkroot)"
        ln -s "$sdkroot_path" "${tmp}/sdk-${target}"
    fi
    printf '%s\n' "$source_dir"
}

run_cross() {
    target="$1"
    source_dir="$2"
    cargo_target_dir="$3"
    (
        cd "$source_dir"
        "$cross_bin" build \
            --target-dir "$cargo_target_dir" \
            --profile capi \
            --target "$target" \
            --lib
    )
}

run_zigbuild() {
    target="$1"
    source_dir="$2"
    cargo_target_dir="$3"
    if is_darwin_target "$target" && [ -L "${tmp}/sdk-${target}" ]; then
        (
            cd "$source_dir"
            SDKROOT="../sdk-${target}" "$zigbuild_bin" zigbuild \
                --target-dir "$cargo_target_dir" \
                --profile capi \
                --target "$target" \
                --lib
        )
    else
        (
            cd "$source_dir"
            "$zigbuild_bin" zigbuild \
                --target-dir "$cargo_target_dir" \
                --profile capi \
                --target "$target" \
                --lib
        )
    fi
}

run_zigbuild_build_std() {
    target="$1"
    source_dir="$2"
    cargo_target_dir="$3"
    rustup component add rust-src --toolchain "$build_std_toolchain"
    (
        cd "$source_dir"
        cargo "+$build_std_toolchain" zigbuild \
            -Z build-std=core \
            --target-dir "$cargo_target_dir" \
            --profile capi \
            --target "$target" \
            --lib
    )
}

archive_target() {
    target="$1"
    cargo_target_dir="$2"
    build_dir="${cargo_target_dir}/${target}/capi"
    stage="${tmp}/archive-${target}"
    archive="${out_dir}/release-${target}.tar.gz"

    mkdir -p "$stage/include" "$stage/lib"
    cp "$project_dir/LICENSE" "$stage/LICENSE"
    cp "$project_dir/README.md" "$stage/README.md"
    cp "$project_dir/CHANGELOG.md" "$stage/CHANGELOG.md"
    cp "$project_dir/include/ja_colloquial.h" "$stage/include/ja_colloquial.h"
    cp "$build_dir/libja_colloquial.a" "$stage/lib/libja_colloquial.a"

    if ! is_musl_target "$target"; then
        suffix="$(dynamic_suffix "$target")"
        cp "$build_dir/libja_colloquial.${suffix}" \
            "$stage/lib/libja_colloquial.${suffix}"
    fi

    (
        cd "$stage"
        tar -cf - LICENSE README.md CHANGELOG.md include lib
    ) | gzip -c >"$archive"
    printf '%s\n' "$archive"
}

for target in $cross_targets; do
    source_dir="$(prepare_source "$target")"
    cargo_target_dir="${target_root}/${target}"
    run_cross "$target" "$source_dir" "$cargo_target_dir"
    archive_target "$target" "$cargo_target_dir"
done

for target in $zigbuild_targets; do
    source_dir="$(prepare_source "$target")"
    cargo_target_dir="${target_root}/${target}"
    if requires_build_std "$target"; then
        run_zigbuild_build_std "$target" "$source_dir" "$cargo_target_dir"
    else
        rustup target add "$target"
        run_zigbuild "$target" "$source_dir" "$cargo_target_dir"
    fi
    archive_target "$target" "$cargo_target_dir"
done
