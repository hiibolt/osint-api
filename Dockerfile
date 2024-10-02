# syntax=docker/dockerfile:1
FROM rust:1.75-slim AS build

RUN apt-get update
RUN apt-get install -y pkg-config curl
RUN apt-get install -y libssl-dev openssl

WORKDIR /
COPY . .

RUN ["cargo", "build", "--release"]

FROM ubuntu
COPY --from=build /target/release/osint-api /osint-api
CMD ["/osint-api"]
ARG PORT
EXPOSE $PORT