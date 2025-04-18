SERVER=botenza.org:8080

tunnel:
	docker run --rm --name=eng-roulette-tunnel --network=host jpillora/chisel:latest -- client $(SERVER) R:8057:5157 R:8081:8081 R:8082:8082

check:
	cargo fmt --all -- --check
	cargo clippy --all-targets --all-features -- -D warnings

test:
	# TODO auto migrate db
	cargo test -p account -p room
