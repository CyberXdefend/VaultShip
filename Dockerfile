FROM rust:latest AS builder
WORKDIR /workspace

# Cache dependencies first for faster rebuilds.
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
RUN cargo build --release -p vaultship-cli

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /workspace/target/release/vaultship /usr/local/bin/vaultship
ENTRYPOINT ["vaultship"]
CMD ["--help"]
