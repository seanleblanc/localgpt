# Use the official Rust image as the base
FROM rust:1.93 as builder

# Set the working directory
WORKDIR /usr/src/myapp

# Copy only the Cargo.toml and Cargo.lock to cache dependencies
COPY Cargo.toml Cargo.lock ./

# Build dependencies only
RUN mkdir src && \
    mkdir ui && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -f target/release/deps/myapp*

# Now copy the source code and build the actual application
COPY ./src ./src
COPY ./ui ./ui

# Build the actual application
RUN cargo build --release

# Final base image (alpine is preferred for smaller size)
FROM debian:buster-slim

# Copy the binary from the builder
COPY --from=builder /usr/src/myapp/target/release/myapp /usr/local/bin/myapp

# Set the entry point for the container
CMD ["myapp"]
