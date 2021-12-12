all: dist/index.html dist/cc0.wasm

dist/index.html: web/* dist
	npm exec nattoppet web/index.ymd > dist/index.html

dist/cc0.wasm: src/* Cargo.toml
	RUSTFLAGS="" cargo build --release --target wasm32-unknown-unknown
	cp target/wasm32-unknown-unknown/release/cc0.wasm dist

dist:
	mkdir -p dist

clean:
	rm -rf dist
	cargo clean
