.PHONY: check
check: githooks
	cargo check

.PHONY: githooks
githooks:
	cp -r scripts/githooks/ .git/hooks/

.PHONY: test
test:
	cargo test -- --nocapture

.PHONY: ci
ci:
	cargo clippy --all --all-targets
	cargo fmt --all -- --check
	cargo test