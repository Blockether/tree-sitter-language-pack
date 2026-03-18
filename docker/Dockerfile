# tree-sitter-language-pack CLI — static musl binary with all 173 parsers
# Build: docker build -f docker/Dockerfile -t ts-pack .
# Run:   docker run --rm ts-pack parse --language python -

# ─── Builder ─────────────────────────────────────────────────────────
FROM rust:alpine AS builder

RUN apk add --no-cache musl-dev gcc g++ python3 py3-pip git bash

WORKDIR /build
COPY . .

# Clone grammar sources if not present (CI may not have them)
RUN if [ ! -d parsers ] || [ -z "$(ls -A parsers 2>/dev/null)" ]; then \
      pip install --break-system-packages gitpython anyio typing-extensions && \
      python3 scripts/clone_vendors.py; \
    fi

# Build CLI with all parsers statically linked
RUN LANGS=$(python3 -c "import json; d=json.load(open('sources/language_definitions.json')); print(','.join(sorted(d.keys())))") && \
    echo "Building with $(echo $LANGS | tr ',' '\n' | wc -l) languages" && \
    TSLP_LANGUAGES="$LANGS" \
    TSLP_LINK_MODE=static \
    PROJECT_ROOT=/build \
    cargo build --release -p ts-pack-cli && \
    strip target/release/ts-pack

# ─── Runtime ─────────────────────────────────────────────────────────
FROM alpine:latest

COPY --from=builder /build/target/release/ts-pack /usr/local/bin/ts-pack

# Smoke test — parsers are statically linked, no downloads needed
RUN ts-pack --version && \
    echo "def hello(): pass" | ts-pack parse - --language python

ENTRYPOINT ["ts-pack"]
