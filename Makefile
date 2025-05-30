install:
	cargo build --release
	sudo install -m755 target/release/shesh /usr/bin/shesh

uninstall:
	sudo rm -f /usr/bin/shesh

clean:
	cargo clean
