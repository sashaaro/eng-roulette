SERVER=botenza.org:8080

tunnel:
	docker run --rm --name=eng-roulette-tunnel --network=host jpillora/chisel:latest -- client $(SERVER) R:8083:8083

check:
	cargo fmt --all -- --check
	cargo clippy --all-targets --all-features -- -D warnings

test:
	# TODO auto migrate db
	cargo test -p account -p room
