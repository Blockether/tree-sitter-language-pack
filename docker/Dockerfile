# tree-sitter-language-pack CLI — static musl binary with all 173 parsers
# Build: docker build -f docker/Dockerfile -t ts-pack .
# Run:   docker run --rm ts-pack parse --language python -

# ─── Builder ─────────────────────────────────────────────────────────
FROM rust:alpine AS builder

RUN apk add --no-cache musl-dev gcc g++ python3 git bash

WORKDIR /build
COPY . .

# Get all language names from definitions
RUN LANGS=$(python3 -c "import json; d=json.load(open('sources/language_definitions.json')); print(','.join(sorted(d.keys())))") && \
    echo "Building with ${LANGS}" && \
    TSLP_LANGUAGES="$LANGS" \
    TSLP_LINK_MODE=static \
    PROJECT_ROOT=/build \
    cargo build --release -p ts-pack-cli && \
    strip target/release/ts-pack

# ─── Runtime ─────────────────────────────────────────────────────────
FROM alpine:latest

COPY --from=builder /build/target/release/ts-pack /usr/local/bin/ts-pack

# Smoke test
RUN ts-pack --version && \
    echo "def hello(): pass" | ts-pack parse - --language python

ENTRYPOINT ["ts-pack"]
