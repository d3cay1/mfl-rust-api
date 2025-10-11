# ---- Builder Stage ----
# Use a Rust image based on Debian Bookworm as a parent image for building
FROM rust:bookworm AS builder
# If you need a slim version and it's available, it might be rust:bookworm-slim
# or a specific version like rust:1.78-bookworm

# Install OpenSSL 3.x development libraries and pkg-config
RUN apt-get update && apt-get install -y libssl-dev pkg-config && rm -rf /var/lib/apt/lists/*

# Set the working directory in the container
WORKDIR /usr/src/app

# Copy the Cargo.toml and Cargo.lock files for the workspace root
COPY Cargo.toml Cargo.lock ./

# Copy your workspace members
COPY mfl_manager ./mfl_manager
COPY mfl_manager_lib ./mfl_manager_lib
COPY integration_test ./integration_test

# Build the application for release
RUN cargo build --manifest-path ./Cargo.toml --release --package mfl_manager

# ---- Runtime Stage ----
# Use Debian Bookworm slim for the final container, which includes OpenSSL 3.x
FROM debian:bookworm-slim

# Set DEBIAN_FRONTEND to noninteractive to prevent any interactive prompts
ENV DEBIAN_FRONTEND=noninteractive

# Update package lists and install libssl3 and ca-certificates
# libssl3 provides libssl.so.3
# ca-certificates is generally needed for HTTPS communication
RUN apt-get update && \
    apt-get install -y --no-install-recommends libssl3 ca-certificates && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/* /tmp/* /var/tmp/*

# Set the working directory
WORKDIR /usr/local/bin

# Copy the compiled binary from the builder stage
COPY --from=builder /usr/src/app/target/release/mfl_manager .

# Expose the port your application listens on
EXPOSE 8080

# Command to run the application
CMD ["./mfl_manager"]
