.PHONY: compose\:up
compose\:up:
	docker compose -f docker-compose.yml -f api/docker-compose.yml up -d

.PHONY: compose\:down\:all
compose\:down\:all:
	docker compose -f docker-compose.yml -f api/docker-compose.yml down --remove-orphans --rmi local --volumes
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