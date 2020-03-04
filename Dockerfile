# Dockerfile for creating a statically-linked Rust application using docker's
# multi-stage build feature. This also leverages the docker build cache to avoid
# re-downloading dependencies if they have not changed.
#FROM registry.gitlab.com/rust_musl_docker/image:stable-latest as build
FROM ekidd/rust-musl-builder:stable-openssl11 as build

ADD --chown=rust:rust . ./
#RUN cargo install --target x86_64-unknown-linux-musl --path .
RUN cargo build --release
# Copy the statically-linked binary into an alpine linux container for execution.
FROM alpine
COPY --from=build /home/rust/src/target/x86_64-unknown-linux-musl/release/highspot /usr/local/bin/

# Since we're using environment variables for config
# We'll setup some values via ARGs then use them in ENVs as defaults
# These can then easily be overriden via a docker-compose.yml or via the docker cli
ARG DATABASE_URL='postgres://user:bad_pass@localhost/webhooks'
ENV DATABASE_URL=$DATABASE_URL

ARG DATABASE_MAX_CONN='20'
ENV DATABASE_MAX_CONN=$DATABASE_MAX_CONN


USER 1000
CMD ["/usr/local/bin/highspot"]
