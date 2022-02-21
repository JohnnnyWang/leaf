ios:
	cargo lipo --release --targets aarch64-apple-ios --manifest-path leaf-ffi/Cargo.toml --no-default-features --features "default-openssl"
	cbindgen --config leaf-ffi/cbindgen.toml leaf-ffi/src/lib.rs > target/universal/release/leaf.h

ios-dev:
	cargo lipo --targets aarch64-apple-ios --manifest-path leaf-ffi/Cargo.toml --no-default-features --features "default-openssl"
	cbindgen --config leaf-ffi/cbindgen.toml leaf-ffi/src/lib.rs > target/universal/debug/leaf.h

lib:
	cargo build --target aarch64-linux-android -p leaf-ffi --release
	cargo build --target x86_64-linux-android -p leaf-ffi --release
	cargo build --target armv7-linux-androideabi -p leaf-ffi --release
	cargo build --target i686-linux-android -p leaf-ffi --release
	cbindgen --config leaf-ffi/cbindgen.toml leaf-ffi/src/lib.rs > target/release/leaf.h
	rm -rf ../VilaVpnClient/android/app/src/main/jniLibs/arm64-v8a/libleaf*.so
	cp target/aarch64-linux-android/release/libleaf.so ../VilaVpnClient/android/app/src/main/jniLibs/arm64-v8a/
	rm -rf ../VilaVpnClient/android/app/src/main/jniLibs/x86_64/libleaf*.so
	cp target/x86_64-linux-android/release/libleaf.so ../VilaVpnClient/android/app/src/main/jniLibs/x86_64/
	rm -rf ../VilaVpnClient/android/app/src/main/jniLibs/armeabi-v7a/libleaf*.so
	cp target/armv7-linux-androideabi/release/libleaf.so ../VilaVpnClient/android/app/src/main/jniLibs/armeabi-v7a/
	rm -rf ../VilaVpnClient/android/app/src/main/jniLibs/x86/libleaf*.so
	cp target/i686-linux-android/release/libleaf.so ../VilaVpnClient/android/app/src/main/jniLibs/x86/

lib-dev:
	cargo build -p leaf-ffi
	cargo build --target aarch64-linux-android -p leaf-ffi
	cargo build --target x86_64-linux-android -p leaf-ffi
	cargo build --target armv7-linux-androideabi -p leaf-ffi
	cargo build --target i686-linux-android -p leaf-ffi
	cbindgen --config leaf-ffi/cbindgen.toml leaf-ffi/src/lib.rs > target/debug/leaf.h

local:
	cargo build -p leaf-bin --release

local-dev:
	cargo build -p leaf-bin

mipsel:
	./misc/build_cross.sh mipsel-unknown-linux-musl

test:
	cargo test -p leaf -- --nocapture

# Force a re-generation of protobuf files.
proto-gen:
	touch leaf/build.rs
	PROTO_GEN=1 cargo build -p leaf

clean:
	rm -rf ../VilaVpnClient/android/app/src/main/jniLibs/arm64-v8a/libleaf*.so