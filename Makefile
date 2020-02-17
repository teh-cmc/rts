.PHONY: desktop web

desktop:
	cargo run --release

web:
	cargo web start --release --use-system-emscripten --host 0.0.0.0
