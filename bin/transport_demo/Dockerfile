# syntax=docker/dockerfile:1

ARG RUST_VERSION=1.92.0

FROM rust:${RUST_VERSION}-alpine AS build
WORKDIR /app

RUN apk add --no-cache clang lld musl-dev git

COPY . .

RUN cargo build --release --bin transport_demo

# Copy binary to final image
FROM alpine:3.18 AS final
WORKDIR /bin
COPY --from=build /app/target/release/transport_demo /bin/server

CMD ["/bin/server"]
