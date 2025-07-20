# Makefile to simplify test commands
.PHONY: test test-unit test-integration test-all clean setup-test-db help

# Variables
TEST_DB_URL := mysql://root:password@localhost:3307/ruggine_test
CARGO := cargo

help: ## Show this help
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-30s\033[0m %s\n", $$1, $$2}'

test: ## Run all tests
	$(CARGO) test

test-unit: ## Run only unit tests
	$(CARGO) test --test "*unit*"

test-integration: ## Run only integration tests
	TEST_DATABASE_URL=$(TEST_DB_URL) $(CARGO) test --test "*integration*" -- --test-threads=1

test-all: test-unit test-integration ## Run unit and integration tests separately

test-verbose: ## Run tests with verbose output
	$(CARGO) test -- --nocapture

test-specific: ## Run a specific test (usage: make test-specific TEST=test_name)
	$(CARGO) test $(TEST) -- --nocapture

setup-test-db: ## Setup test database with Docker
	docker-compose -f docker-compose.test.yml up -d
	sleep 10
	sqlx migrate run --database-url=$(TEST_DB_URL)

teardown-test-db: ## Remove test database
	docker-compose -f docker-compose.test.yml down -v

clean: ## Clean build files
	$(CARGO) clean

coverage: ## Generate coverage report
	$(CARGO) tarpaulin --out Html

lint: ## Run code quality checks
	$(CARGO) clippy -- -D warnings
	$(CARGO) fmt --check

fix: ## Automatically fix formatting issues
	$(CARGO) fmt
	$(CARGO) clippy --fix

pre-commit: test-unit lint ## Pre-commit checks
	@echo "✅ Pre-commit checks passed!"

pre-merge: test-all coverage lint ## Pre-merge checks
	@echo "✅ Pre-merge checks passed!"

watch-test: ## Run tests in watch mode
	$(CARGO) watch -x test

benchmark: ## Run performance benchmarks
	$(CARGO) bench

# CI/CD targets
ci-test: setup-test-db test-all teardown-test-db ## Run complete tests for CI

# Development helpers
dev-setup: ## Setup development environment
	rustup component add clippy rustfmt
	cargo install cargo-watch cargo-tarpaulin sqlx-cli

docs: ## Generate documentation
	$(CARGO) doc --open
