FROM rust:slim-bullseye as build

RUN apt-get update
RUN apt-get install pkg-config -y
RUN apt-get install openssl -y
RUN apt-get install libssl-dev -y
RUN apt-get install build-essential -y

RUN USER=root cargo new --lib /app/build/ext-server
RUN USER=root cargo new --bin /app/build/ext-server/basic-impl

WORKDIR /app/build/ext-server

COPY basic-impl/Cargo.toml ./basic-impl/Cargo.toml
COPY basic-impl/Cargo.lock ./basic-impl/Cargo.lock

COPY Cargo.lock ./
COPY Cargo.toml ./

RUN cd basic-impl && cargo build --release

RUN rm src/*.rs
RUN rm basic-impl/src/*.rs

COPY src ./src
COPY basic-impl/src ./basic-impl/src

RUN cargo install --path basic-impl --target-dir /app/bin

FROM debian:bullseye-slim

COPY --from=build /app/bin/release/basic-impl /app/bin/basic-impl

RUN apt-get update
RUN apt-get install pkg-config -y
RUN apt-get install openssl -y
RUN apt-get install libssl-dev -y
RUN apt-get install apt-transport-https ca-certificates gnupg curl -y

EXPOSE $PORT

WORKDIR /app/data
ENTRYPOINT ["/app/bin/basic-impl"]