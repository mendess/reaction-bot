# inspiration: https://dev.to/rogertorres/first-steps-with-docker-rust-30oi

FROM rust:1.70.0-buster as build

RUN apt update -y && apt install cmake libpython3-dev -y

# create an empty shell project
RUN USER=root cargo new --bin reaction-bot
WORKDIR /reaction-bot

# copy manifests
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

# cache dependencies
RUN cargo build --release --workspace
RUN rm -r ./src

# copy real source
COPY ./src ./src

# build for release
RUN rm ./target/release/reaction-bot
RUN find ./src -name '*.rs' -exec touch '{}' ';'
RUN cargo build --release --workspace

# executing image
FROM debian:buster-slim

RUN apt update -y && apt install libopus-dev ffmpeg youtube-dl libpython3-dev -y

COPY --from=build /reaction-bot/target/release/reaction-bot .

ENTRYPOINT ["./reaction-bot"]
