all: dist/index.html

dist/index.html: web/* dist target/wasm32-unknown-unknown/release/cc0.wasm
	npm exec nattoppet web/index.ymd > dist/index.html

target/wasm32-unknown-unknown/release/cc0.wasm: src/* Cargo.toml
	RUSTFLAGS="" cargo build --release -Z build-std=core,alloc --target wasm32-unknown-unknown

dist:
	mkdir -p dist

clean:
	rm -rf dist
	cargo clean
