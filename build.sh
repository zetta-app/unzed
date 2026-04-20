#!/usr/bin/env bash
set -euo pipefail

# Local build script. For glibc-compatible release builds, use the GitHub
# Actions workflow (.github/workflows/build_custom_release.yml) which builds
# on Ubuntu 22.04 (glibc 2.35).

export CC=${CC:-$(which clang)}

arch=$(uname -m)
dist_dir="dist/release"

export RUSTFLAGS="${RUSTFLAGS:-} -C link-args=-Wl,--disable-new-dtags,-rpath,\$ORIGIN/../lib,-rpath,\$ORIGIN/lib"

if [[ "$arch" == "aarch64" ]]; then
    export RUSTFLAGS="${RUSTFLAGS} -C link-arg=-fuse-ld=lld"
fi

# Build with only qwen and openai-compatible providers.
cargo build -p zed -p cli --release --no-default-features --features provider-open-ai

# Package into dist/release
rm -rf "${dist_dir}"
mkdir -p "${dist_dir}/lib" "${dist_dir}/bin"

cp target/release/zed "${dist_dir}/libexec-zed"
cp target/release/cli "${dist_dir}/bin/zed"

# Bundle shared libraries (excluding base system libs)
ldd target/release/zed |
    awk '/=>/ { print $3 }' |
    grep -v '^\s*$' |
    grep -v '\(libc\.so\|libm\.so\|libpthread\|libdl\|ld-linux\)' |
    while read -r lib; do
        [ -f "$lib" ] && cp "$lib" "${dist_dir}/lib/"
    done

llvm-objcopy --strip-debug "${dist_dir}/libexec-zed"
llvm-objcopy --strip-debug "${dist_dir}/bin/zed"

# Wrapper script that sets LD_LIBRARY_PATH for bundled libs
cat > "${dist_dir}/zed-editor" << 'WRAPPER'
#!/usr/bin/env bash
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
export LD_LIBRARY_PATH="${SCRIPT_DIR}/lib${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
exec "${SCRIPT_DIR}/libexec-zed" "$@"
WRAPPER
chmod +x "${dist_dir}/zed-editor"

echo "Release files written to ${dist_dir}/"
echo "Run with: ${dist_dir}/zed-editor"
