cargo build -p zed --release
llvm-objcopy --strip-debug target/release/zed
cp target/release/zed zed