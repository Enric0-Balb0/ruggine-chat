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

# Docker targets
docker-build: ## Build Docker image for production
	docker-compose build

docker-prod: ## Start production environment with Docker
	docker-compose up -d

docker-dev: ## Start development environment with Docker
	docker-compose -f docker-compose.dev.yml up -d

docker-logs: ## Follow Docker logs for production
	docker-compose logs -f

docker-logs-dev: ## Follow Docker logs for development
	docker-compose -f docker-compose.dev.yml logs -f

docker-stop: ## Stop all Docker services
	docker-compose down
	docker-compose -f docker-compose.dev.yml down

docker-clean: ## Stop and remove all Docker containers and volumes
	docker-compose down -v
	docker-compose -f docker-compose.dev.yml down -v

docker-rebuild: ## Force rebuild and restart production environment
	docker-compose down
	docker-compose up -d --build

docker-shell: ## Get shell access to running production container
	docker-compose exec ruggine_server sh

docker-shell-dev: ## Get shell access to running development container
	docker-compose -f docker-compose.dev.yml exec ruggine_dev_server sh

docker-db: ## Access production database shell
	docker-compose exec ruggine_db psql -U ruggine_user -d ruggine_prod

docker-db-dev: ## Access development database shell
	docker-compose -f docker-compose.dev.yml exec ruggine_dev_db psql -U ruggine_dev_user -d ruggine_dev

docker-status: ## Show status of all Docker containers
	docker-compose ps
	docker-compose -f docker-compose.dev.yml ps

# Quick deployment targets
deploy-prod: ## Deploy to production (build and start)
	@echo "🚀 Deploying to production..."
	docker-compose up -d --build
	@echo "✅ Production deployment complete!"
	@echo "Server: http://localhost:8002"

deploy-dev: ## Deploy to development (build and start)
	@echo "🚀 Deploying to development..."
	docker-compose -f docker-compose.dev.yml up -d --build
	@echo "✅ Development deployment complete!"
	@echo "Server: http://localhost:8003"
	@echo "pgAdmin: http://localhost:8080"

# Docker health and monitoring
docker-health: ## Check health of Docker services
	docker-compose ps
	docker inspect ruggine_app_server | grep -A 5 Health || echo "No health check configured"

docker-stats: ## Show Docker container resource usage
	docker stats ruggine_app_server ruggine_database

# Backup and restore (production)
backup-db: ## Backup production database
	docker-compose exec ruggine_db pg_dump -U ruggine_user ruggine_prod > backup_$(shell date +%Y%m%d_%H%M%S).sql
	@echo "✅ Database backup created"

restore-db: ## Restore production database (usage: make restore-db BACKUP=backup_file.sql)
	@if [ -z "$(BACKUP)" ]; then echo "❌ Please specify BACKUP=backup_file.sql"; exit 1; fi
	docker-compose exec -T ruggine_db psql -U ruggine_user ruggine_prod < $(BACKUP)
	@echo "✅ Database restored from $(BACKUP)"
