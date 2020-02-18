.PHONY: desktop web

desktop:
	cargo run --release --features parallel

web:
	cargo web start --release --use-system-emscripten --host 0.0.0.0

web-deploy:
	cargo web deploy --release --use-system-emscripten
