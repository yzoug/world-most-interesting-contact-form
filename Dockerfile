# inspired by https://hub.docker.com/_/rust#how-to-use-this-image
MAINTAINER zoug <git@zoug.fr>
FROM rust:1.91 AS builder

WORKDIR /usr/src/axum-backend
COPY . .
RUN cargo install --path .

FROM debian:trixie-slim

RUN <<EOF
apt-get update
apt-get install -y ca-certificates
rm -rf /var/lib/apt/lists/*
EOF

COPY --from=builder /usr/local/cargo/bin/axum-backend /usr/local/bin/axum-backend
CMD ["axum-backend"]
