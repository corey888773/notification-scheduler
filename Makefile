.PHONY: compose\:up
compose\:up:
	docker compose -f docker-compose.yml -f api/docker-compose.yml up -d

.PHONY: compose\:down\:all
compose\:down\:all:
	docker compose -f docker-compose.yml -f api/docker-compose.yml down --remove-orphans --rmi local --volumes