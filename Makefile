.PHONY: desktop web

desktop:
	RUST_BACKTRACE=1 cargo run --verbose --release --features parallel
	# RUST_BACKTRACE=1 cargo run --verbose --features parallel

web:
	RUST_BACKTRACE=1 cargo web start --verbose --release --use-system-emscripten --host 0.0.0.0
	# RUST_BACKTRACE=1 cargo web start --verbose --use-system-emscripten --host 0.0.0.0

web-deploy:
	cargo web deploy --release --use-system-emscripten
