.PHONY: all build run clean

all: build

build:
	wasm-pack build --target web --out-dir pkg

run:
	@fuser -k 8000/tcp 2>/dev/null || true; python3 server.py

clean:
	rm -rf pkg
