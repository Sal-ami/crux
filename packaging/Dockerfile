# syntax=docker/dockerfile:1
FROM rust:1.83-slim AS build
WORKDIR /src
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release --bin crux

FROM gcr.io/distroless/cc-debian12
COPY --from=build /src/target/release/crux /usr/local/bin/crux
ENTRYPOINT ["crux"]
