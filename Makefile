.PHONY: compose\:up\:stack
compose\:up\:stack:
	docker compose -f docker-compose.yml up mongodb nats -d

.PHONY: compose\:up\:build
compose\:up\:build:
	docker compose -f docker-compose.yml build

.PHONY: compose\:up\:run
compose\:up\:run:
	docker compose -f docker-compose.yml up api email_consumer1 email_consumer2 push_consumer1 push_consumer2

.PHONY: compose\:down\:all
compose\:down\:all:
	docker compose -f docker-compose.yml down --remove-orphans --rmi local --volumes
	rm -rf .image_resources/

.PHONY: api\:run
api\:run:
	cargo run --bin api

.PHONY: email\:run
email\:run:
	cargo run --bin email_consumer

.PHONY: fmt
fmt:
	cargo clippy --all-targets --all-features --fix --allow-dirty --allow-staged
	cargo fmt --all
	cargo fix --allow-dirty --allow-staged --edition-idioms --workspace