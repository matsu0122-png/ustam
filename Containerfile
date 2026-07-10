# Containerfile
#
# Multi-stage build: compile a statically linked (musl) release binary,
# then copy only that binary into a distroless, shell-less, non-root
# runtime image. Keeps the shipped image minimal and avoids bundling
# the Rust toolchain in the final artifact.

FROM rust:1-alpine AS builder

RUN apk add --no-cache musl-dev

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo build --release --locked

FROM gcr.io/distroless/static-debian12:nonroot AS runtime

ARG VERSION=0.0.0
ARG GIT_REVISION=unknown
ARG BUILD_DATE=unknown

LABEL org.opencontainers.image.title="ustam" \
      org.opencontainers.image.description="Rust CLI tool inspired by ls" \
      org.opencontainers.image.version="${VERSION}" \
      org.opencontainers.image.revision="${GIT_REVISION}" \
      org.opencontainers.image.created="${BUILD_DATE}" \
      org.opencontainers.image.licenses="MIT" \
      org.opencontainers.image.source="https://github.com/matsu0122-png/ustam" \
      org.opencontainers.image.authors="matsu0122-png"

COPY --from=builder /app/target/release/ustam /usr/local/bin/ustam

USER nonroot:nonroot
WORKDIR /workspace

ENTRYPOINT ["/usr/local/bin/ustam"]
