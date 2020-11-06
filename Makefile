compose = docker-compose

.PHONY: all
all:
	cargo build --release

.PHONY: check
check:
	cargo check
	cargo fmt --all -- --check
	cargo clippy --jobs 4

.PHONY: fmt
fmt:
	cargo fmt

.PHONY: prepare
prepare: fmt check

.PHONY: integration_tests
integration_tests:
	$(compose) -f integration_tests.yml up --build --abort-on-container-exit --exit-code-from integration_tests

.PHONY: clean
clean:
	$(compose) -f integration_tests.yml down
	cargo clean
