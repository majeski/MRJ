RUST_PATH=./cargo/rustc/bin/rustc
CARGO_PATH=./cargo/target/x86_64-*/release/cargo
BIN_PATH=./target/release/instant

# only for students server
all: cargo
	RUSTC=$(RUST_PATH) $(CARGO_PATH) build --release --features "jvm"
	cp $(BIN_PATH) insc_jvm
	RUSTC=$(RUST_PATH) $(CARGO_PATH) build --release --features "llvm"
	cp $(BIN_PATH) insc_llvm

jvm:
	cargo build --release --features "jvm"
	cp $(BIN_PATH) insc_jvm

llvm:
	cargo build --release --features "llvm"
	cp $(BIN_PATH) insc_llvm

# only for students server
cargo:
	git clone --recursive https://github.com/rust-lang/cargo &&\
	cd cargo &&\
	python -B src/etc/install-deps.py &&\
	./configure --local-rust-root=rustc &&\
	make &&\
	cd ..

clean:
	- cargo clean
	rm -rf insc_{jvm,llvm}
# only for students server
	- $(CARGO_PATH) clean
	rm -rf cargo
