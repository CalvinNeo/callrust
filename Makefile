FORCE: ; 

gen: FORCE
	cd gen && cargo run

rustpeer: FORCE
	cd rustpeer && cargo build

cpeer: FORCE
	cd cpeer && g++ --std=c++17 cpeer.cpp ffi.cpp -L../rustpeer/target/debug -lrustpeer  -o a

test:
	cd rustpeer && cargo test

build: cpeer

fmt: FORCE
	cd rustpeer && cargo fmt