FROM rust:1.70.0-slim as builder

WORKDIR /usr/src/notification

# Create blank project
RUN USER=root cargo new medium-rust-dockerize

RUN mkdir -p /usr/src/common

COPY ./common ../common

# We want dependencies cached, so copy those first.
COPY ./notification/Cargo.toml ./notification/Cargo.lock /usr/src/notification/

# # Set the working directory
# WORKDIR /usr/src/medium-rust-dockerize

## Install target platform (Cross-Compilation) --> Needed for Alpine
RUN rustup target add x86_64-unknown-linux-musl

# This is a dummy build to get the dependencies cached.
RUN cargo build --target x86_64-unknown-linux-musl --release

# Now copy in the rest of the sources
COPY ./notification/src /usr/src/notification/src/

## Touch main.rs to prevent cached release build
# RUN touch /usr/src/notification/src/main.rs

# This is the actual application build.
RUN cargo build --target x86_64-unknown-linux-musl --release
# ------------------------------------------------------------------------------
# Cargo Build Stage
# ------------------------------------------------------------------------------
# FROM rust:alpine as cargo-build
#
# WORKDIR /usr/src/notification
# RUN apk update && \
#     apk upgrade
# RUN apk add protoc protobuf-dev
# RUN apk add build-base
# RUN apk add clang llvm
# RUN apk add openssl openssl-dev 
# RUN apk add pkgconfig
# RUN apk add --no-cache musl-dev
# RUN rustup target add x86_64-unknown-linux-musl
#
# RUN mkdir -p /usr/src/common
# COPY ./common ../common
# COPY ./notification .
#
# RUN RUSTFLAGS="-Ctarget-feature=-crt-static" cargo build --release --target=x86_64-unknown-linux-musl
# RUN RUSTFLAGS="-Ctarget-feature=-crt-static" cargo install --path .
#
# ------------------------------------------------------------------------------
# Final Stage
# ------------------------------------------------------------------------------

# FROM alpine:latest
#
# COPY --from=cargo-build /usr/local/cargo/bin/notification /usr/local/bin/notification
#
# CMD ["notification"]


################
##### Runtime
FROM alpine:3.16.0 AS runtime 

# Copy application binary from builder image
COPY --from=builder /usr/src/notification/target/x86_64-unknown-linux-musl/release/notification /usr/local/bin

# EXPOSE 3030

# Run the application
CMD ["/usr/local/bin/notification"]
