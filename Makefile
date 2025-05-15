# cargo install cargo-watch
dev:
	cargo watch -x check -x test -x run

fmt:
	cargo fmt

check: fmt
	cargo check

PATTERN?="update_db"
test: check
	cargo test ${PATTERN}

test-verbose: check
	cargo test -- --nocapture

# cargo install cargo-tarpaulin
cov:
	cargo tarpaulin --ignore-tests

# rustup component add clippy
lint-check: check
	cargo clippy -- -D warnings

# rustup component add rustfmt, for CI pipeline
fmt-check:
	cargo fmt -- --check

# cargo install cargo-audit
audit:
	cargo audit

# cargo install cargo-deny
# equivalent to cargo-audit
deny-audit:
	cargo deny

build:
	cargo build

# cargo install cargo-asm
asm:
	cargo asm

# cargo install bunyan
test-log:
	export RUST_LOG="sqlx=error,info"
	export TEST_LOG=true
	cargo test ${PATTERN} | bunyan

show-todos:
	grep -rni ./crates -e 'todo'

# slqx cli is requied
# cargo install --version="~0.7" sqlx-cli --no-default-features \
  --features rustls,postgres
# Expose databse url for local developement
# export DATABASE_URL=postgres://postgres:password@127.0.0.1:5432/did_demo

init-db:
	./scripts/init_db.sh

init-sqlx-offline:
	export DATABASE_URL=postgres://postgres:password@127.0.0.1:5432/did_demo
	cargo sqlx prepare --workspace

MIGRATION?="update_db"
add-migration:
	sqlx migrate add $(MIGRATION)

# Required to setup docker first
run-migration:
	SKIP_DOCKER=true ./scripts/init_db.sh

# cargo install cargo-udeps
scan:
	cargo +nightly udeps

docker-build:
	docker build --tag did_demo --file Dockerfile .

docker-run:
	docker run -p 8000:8000 did_demo

# JWT
jwt-keypair:
	./scripts/init_jwt_keypair.sh

# This command is used to fix the error
# thread 'actix-rt:worker' panicked at
# 'Can not create Runtime: Os { code: 24, kind: Other, message: "Too many open files" }',
#  limit enforced by the operating system on the maximum number of open file descriptors
# (including sockets) for each process
extend-open-files:
	ulimit -n 1000

cargo-bench:
	cargo +nightly bench
