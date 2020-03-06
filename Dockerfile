# Dockerfile for creating a statically-linked Rust application using docker's
# multi-stage build feature allowing for very small final containers
FROM ekidd/rust-musl-builder:stable-openssl11 as build

ADD --chown=rust:rust . ./
RUN cargo build --release
# Copy the statically-linked binary into a slimmed down container for execution.
# Use alpine if you need in-container debugging
# FROM alpine
FROM scratch
COPY --from=build /home/rust/src/target/x86_64-unknown-linux-musl/release/hook-recorder /usr/local/bin/

# Since we're using environment variables for config
# We'll setup some values via ARGs then use them in ENVs as defaults
# These can then easily be overriden via a docker-compose.yml or via the docker cli
ARG DATABASE_URL='postgres://user:bad_pass@localhost/webhooks'
ENV DATABASE_URL=$DATABASE_URL

ARG DATABASE_MAX_CONN='20'
ENV DATABASE_MAX_CONN=${DATABASE_MAX_CONN}

ARG LISTEN_PORT='3030'
ENV LISTEN_PORT=${LISTEN_PORT}

ARG LISTEN_IP='0.0.0.0'
ENV LISTEN_IP=${LISTEN_IP}

ARG STATS_INTERVAL='20'
ENV STATS_INTERVAL=${STATS_INTERVAL}


USER 1000
CMD ["/usr/local/bin/hook-recorder"]
