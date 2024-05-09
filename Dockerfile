# Docker recipe (Dockerfile)
# `docker build --tag zero2prod --file Dockerfile .` generates an image based on this recipe.
# Using `.` tells docker to use the current directory as the build context.

# NOTE: Get cargo chef 

# We use the latest stable release as base image
# FROM rust:1.78.0 AS builder
FROM lukemathwalker/cargo-chef:latest-rust-1 as chef
# Switch working directory to `app` (equivalent to `cd app`)
# The `app` folder will be created by docker if it doesn't exist.
WORKDIR /app
# Install the required system dependencies for our linking configuration
RUN apt update && apt install lld clang -y

# NOTE: Planner stage

FROM chef as planner
# Copy all files from our working environment to the Docker image
COPY . .
# Compute a lock-like file for the project
RUN cargo chef prepare --recipe-path recipe.json

# NOTE: Builder stage

FROM chef as builder
COPY --from=planner /app/recipe.json recipe.json
# Building project deps, not the app!
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
# Forces SQLX verification at compile time to use the cached results from `sqlx-prepare`
ENV SQLX_OFFLINE true
# Build the binary with the release profile
RUN cargo build --release --bin zero2prod

# NOTE: Runtime stage

FROM debian:bookworm-slim AS runtime
WORKDIR /app
# Install OpenSSL - it is dynamically linked by some of our dependencies
# Install ca-certificates - it is needed to verity TLS certificates when establishing HTTP connections
RUN apt-get update -y \
    && apt-get install -y --no-install-recommends openssl ca-certificates \
    # Clean up
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*

# Copy the compiled binary from the builder environment to the runtime environment
COPY --from=builder /app/target/release/zero2prod zero2prod 

# We need the configuration file at runtime
COPY configuration configuration

# Use production env
ENV APP_ENVIRONMENT production

# When `docker run` is executed, launch the binary!
ENTRYPOINT [ "./zero2prod" ]

