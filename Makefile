FORCE: ; 

gen: FORCE
	cd gen && cargo run

rustpeer: gen
	cd rustpeer
	cargo build

build: rustpeer