.PHONY: all build release test c-test clean install

all: build

build:
	cargo build

release:
	cargo build --release

test:
	cargo test --all
	$(MAKE) c-test

c-test: release
	gcc -O2 -Iinclude tests/test_c_ffi.c target/release/libblks_core.a -lpthread -ldl -lm -o tests/test_c_ffi
	./tests/test_c_ffi

clean:
	cargo clean
	rm -f tests/test_c_ffi

install: release
	install -Dm755 target/release/blks /usr/local/bin/blks
	install -Dm644 include/blks.h /usr/local/include/blks.h
	install -Dm644 target/release/libblks_core.a /usr/local/lib/libblks_core.a
