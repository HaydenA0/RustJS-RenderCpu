.PHONY: all build run clean

all: build

build:
	wasm-pack build --target web --out-dir pkg

run:
	python3 server.py

clean:
	rm -rf pkg
