# Stage 0: build planner (cache build plan)
#FROM rust:1.93-slim AS chef
FROM lukemathwalker/cargo-chef:latest-rust-1.93 AS chef
WORKDIR /app


FROM chef AS chef-planner
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates git && rm -rf /var/lib/apt/lists/*
COPY Cargo.toml Cargo.lock ./
# copy minimal src to let cargo-chef analyze deps
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo chef prepare --recipe-path recipe.json


# Stage 1: build dependencies
FROM chef AS chef-cook
COPY --from=chef-planner /app/recipe.json ./
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release --bin localgpt


# Final stage: runtime
FROM gcr.io/distroless/cc-debian13 AS runtime
COPY --from=chef-cook /app/target/release/localgpt /usr/local/bin/localgpt
ENTRYPOINT ["/usr/local/bin/localgpt"]
