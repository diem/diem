all:
	bindgen data.h --whitelist-type=CEventHandle --whitelist-type=CDevAccountResource -o src/data.rs
	cargo build
	cc -flto --std=c11 -fno-omit-frame-pointer -o a.out main.c  -L target/debug -llibra_dev -framework Security
	./a.out