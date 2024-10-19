FROM rust:latest AS builder
#
RUN apt update && apt install -y git musl-tools musl-dev libzstd-dev
RUN update-ca-certificates
#
WORKDIR /opt/honeyaml
COPY . .
#
# Need to dynamically link, otherwise multi platform builds are breaking with zstd-sys
RUN cargo build --release && \
    cp target/release/honeyaml /opt/honeyaml/
#
# Using wolfi instead of ubuntu because of smaller footprint (and required full glibc support)
FROM chainguard/wolfi-base:latest
#
COPY --from=builder /opt/honeyaml/honeyaml /opt/honeyaml/
COPY --from=builder /opt/honeyaml/api.yml /opt/honeyaml/
#
RUN <<EOF
apk update
apk add libstdc++
EOF
#
STOPSIGNAL SIGINT
WORKDIR /opt/honeyaml
CMD ["./honeyaml", "-d", "/opt/honeyaml/log"]
