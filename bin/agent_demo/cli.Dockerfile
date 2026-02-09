ARG RUST_VERSION=1.92.0

FROM rust:${RUST_VERSION}-alpine AS build
WORKDIR /app

RUN apk add --no-cache clang lld musl-dev git

COPY . .

RUN cargo build --release --bin cli

# Final stage
FROM debian:bookworm-slim AS final
WORKDIR /bin

RUN apt-get update && apt-get install -y \
    bash \
    bash-completion \
    less \
    curl \
    wget \
    iputils-ping \
    jq \
    ca-certificates \
    gnupg \
    vim \
    lsb-release \
    python3 \
    python3-venv \
    && rm -rf /var/lib/apt/lists/* \
    && python3 -m venv /opt/venv \
    && /opt/venv/bin/pip install --no-cache-dir httpie

COPY --from=build /app/target/release/cli /bin/cli

ENV PATH="/opt/venv/bin:$PATH"

CMD ["/bin/bash"]
