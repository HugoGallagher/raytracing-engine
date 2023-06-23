cargo build --release
del engine.exe
cd target/release
copy engine.exe "../../engine.exe"