# Use the official Rust image as the base
FROM lukemathwalker/cargo-chef:latest-rust-1.93 as chef
WORKDIR /app

FROM chef as planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef as builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

COPY . .
RUN cargo build --release --bin localgpt


FROM gcr.io/distroless/cc-debian13 AS runtime
WORKDIR /app
COPY --from=builder /app/target/release/localgpt /
ENTRYPOINT ["/localgpt"]

