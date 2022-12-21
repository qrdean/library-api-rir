FROM lukemathwalker/cargo-chef:latest-rust-1.66.0 as chef
WORKDIR /app
# RUN apt update && apt install lld clang -y

FROM chef as planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef as builder
COPY --from=planner /app/recipe.json recipe.json

# build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json

COPY . .
# Target the .json metadata instead of live database
ENV SQLX_OFFLINE TRUE
# Let's build our binary!
# We'll use the release profile to make it faaaast
RUN cargo build --release --bin library-api-rir

FROM debian:bullseye-slim AS runtime
WORKDIR app
COPY --from=builder /app/target/release/library-api-rir /usr/local/bin
ENTRYPOINT ["/usr/local/bin/library-api-rir"]
