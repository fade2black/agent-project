ARG RUST_VERSION=1.92.0

FROM rust:${RUST_VERSION}-alpine AS build
WORKDIR /app

RUN apk add --no-cache clang lld musl-dev git

COPY . .

RUN cargo build --release --bin agent_demo

# Copy binary to final image
FROM alpine:3.18 AS final
WORKDIR /bin
COPY --from=build /app/target/release/agent_demo /bin/server

CMD ["/bin/server"]
