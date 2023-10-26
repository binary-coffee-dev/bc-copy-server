FROM rust:1.70 AS build-container

RUN apt update && apt install libpq5 libssl-dev ca-certificates -y

# setup dummie projet
RUN USER=root cargo new build_dir
WORKDIR /build_dir

# coping and installing the dependencies
COPY Cargo.toml Cargo.lock ./
RUN cargo fetch

# coping and build base code
COPY src ./src
COPY templates ./templates
RUN cargo build --release

FROM debian:buster-slim

WORKDIR /
RUN mkdir data config

ENV CONFIG_PATH="/config"
ENV DATA_PATH="/data"

COPY --from=build-container /build_dir/target/release/copy_service .

EXPOSE 4000
EXPOSE 4001

CMD ["./copy_service"]
