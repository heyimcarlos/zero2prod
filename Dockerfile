# Docker recipe (Dockerfile)
# `docker build --tag zero2prod --file Dockerfile .` generates an image based on this recipe.
# Using `.` tells docker to use the current directory as the build context.

# We use the latest stable release as base image
FROM rust:1.78.0

# Switch working directory to `app` (equivalent to `cd app`)
# The `app` folder will be created by docker if it doesn't exist.
WORKDIR /app

# Install the required system dependencies for our linking configuration
RUN apt update && apt install lld clang -y

# Copy all files from our working environment to the Docker image
COPY . .

# Forces SQLX verification at compile time to use the cached results from `sqlx-prepare`
ENV SQLX_OFFLINE=true

# Build the binary with the release profile
RUN cargo build --release

# Use production env
ENV APP_ENVIRONMENT=production

# When `docker run` is executed, launch the binary!
ENTRYPOINT [ "./target/release/zero2prod" ]

