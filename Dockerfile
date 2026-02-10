# Stage 0: build planner (cache build plan)
FROM rust:1.93-slim as chef-planner
WORKDIR /app
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates git && rm -rf /var/lib/apt/lists/*
COPY Cargo.toml Cargo.lock ./
# copy minimal src to let cargo-chef analyze deps
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo install cargo-chef --version 0.1.42 --locked

# produce recipe
RUN cargo chef prepare --recipe-path recipe.json

# Stage 1: build dependencies
FROM rust:1.93-slim as chef-cook
WORKDIR /app
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates build-essential pkg-config libssl-dev git && rm -rf /var/lib/apt/lists/*
COPY --from=chef-planner /usr/local/cargo/bin/cargo-chef /usr/local/cargo/bin/cargo-chef
COPY --from=chef-planner /app/Cargo.toml /app/Cargo.lock /app/recipe.json ./
RUN cargo chef cook --release --recipe-path recipe.json

# Stage 2: compile application
FROM rust:1.93-slim as builder
WORKDIR /app
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates build-essential pkg-config libssl-dev git && rm -rf /var/lib/apt/lists/*
COPY --from=chef-cook /app/target target
COPY . .
RUN cargo build --release --bin localgpt

# Final stage: runtime
FROM gcr.io/distroless/cc-debian13 as runtime
COPY --from=builder /app/target/release/localgpt /usr/local/bin/localgpt
ENTRYPOINT ["/usr/local/bin/localgpt"]
