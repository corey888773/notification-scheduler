# Build stage
FROM rust:1.86-slim as builder

# Set the working directory.
WORKDIR /usr/src/app

# Copy workspace manifest files first to leverage Docker cache.
# This includes the root Cargo.toml and Cargo.lock.
COPY Cargo.toml Cargo.lock ./
COPY api api/
COPY common common/
COPY push_consumer push_consumer/
COPY email_consumer email_consumer/

# Now copy the full source code.
# Build the API binary in release mode. Adjust package name if needed.
RUN cargo build --release -p push_consumer

FROM debian:stable-slim AS runtime
# Set working directory.
WORKDIR /usr/local/bin

# Copy the built binary from the builder stage.
COPY --from=builder /usr/src/app/target/release/push_consumer .

# Expose the port your API listens on.
EXPOSE 8080
# Run the API binary.
CMD ["./push_consumer"]