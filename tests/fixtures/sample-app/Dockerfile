FROM rust:1.85 AS builder
WORKDIR /app
COPY . .
RUN cargo build --release && strip target/release/sample-app

FROM gcr.io/distroless/cc-debian12:nonroot
COPY --from=builder /app/target/release/sample-app /app/sample-app
USER nonroot:nonroot
EXPOSE 8080
ENTRYPOINT ["/app/sample-app"]
