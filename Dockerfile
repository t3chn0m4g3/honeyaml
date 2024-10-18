FROM rust:latest AS builder
#
# Detect architecture and set target accordingly
RUN arch=$(arch) && \
    if [ "$arch" = "x86_64" ]; then \
        rustup target add x86_64-unknown-linux-musl; \
        echo "TARGET_ARCH=x86_64-unknown-linux-musl" > /target_arch.env; \
    elif [ "$arch" = "aarch64" ]; then \
        rustup target add aarch64-unknown-linux-musl; \
        echo "TARGET_ARCH=aarch64-unknown-linux-musl" > /target_arch.env; \
    else \
        echo "Unsupported architecture: $arch"; \
        exit 1; \
    fi
#
RUN apt update && apt install -y musl-tools musl-dev
RUN update-ca-certificates
#
# Copy source files and set working directory
COPY . .
#
# Load target architecture from file and build
RUN . /target_arch.env && \
    cargo build --target "$TARGET_ARCH" --release && \
    cp /target/"$TARGET_ARCH"/release/honeyaml /
#
# Final stage
FROM scratch
#
WORKDIR /honeyaml
#
# Copy our build
COPY --from=builder /honeyaml ./
COPY --from=builder /api.yml ./
#
CMD ["/honeyaml/honeyaml", "-d", "/honeyaml/logs"]
