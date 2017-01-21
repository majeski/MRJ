all:
	cargo build --release
	cp ./target/release/latte latc
	cp ./target/release/latte latc_llvm

clean:
	cargo clean
	rm -rf target
	rm -rf Cargo.lock
	rm -rf latc latc_llvm
