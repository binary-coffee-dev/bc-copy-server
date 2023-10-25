FROM rust:1.70 AS build-container

# setup dummie projet
RUN USER=root cargo new build_dir
WORKDIR /build_dir

# coping and installing the dependencies
COPY Cargo.toml Cargo.lock ./
RUN cargo fetch

# coping and build base code
COPY src ./src
COPY templates ./templates
RUN cargo build

FROM debian:buster-slim

COPY --from=build-container /build_dir/target/debug/copy_service .

RUN apt update && apt install libssl-dev ca-certificates wget -y

CMD ["./copy_service"]
