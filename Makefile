move: complie
	mv ./target/release/project3-bgp-router ./4700router

complie: client
	~/.cargo/bin/cargo build -r

# Thanks for Luke Jianu
client: 
	curl https://sh.rustup.rs -sSf | sh -s -- -y \
	&& ~/.cargo/bin/rustup install --profile=minimal 1.75.0 \
	&& ~/.cargo/bin/rustup default 1.75.0 