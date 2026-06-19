INPUT = entrada.txt

build:
	cargo build --release

run:
	cargo run --release $(INPUT)
