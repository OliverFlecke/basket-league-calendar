# FROM ubuntu:latest AS base
FROM debian:bullseye-slim AS base

WORKDIR /app
RUN apt-get update
RUN apt-get install wget firefox-esr -y
RUN wget https://github.com/mozilla/geckodriver/releases/download/v0.24.0/geckodriver-v0.24.0-linux64.tar.gz \
  && tar -xvzf geckodriver* \
  && chmod +x geckodriver \
  && rm *.gz

FROM rust:1.72.1-slim-bullseye AS build

WORKDIR /app
COPY Cargo.lock .
COPY Cargo.toml .
RUN mkdir src && echo "// Dummy file" > src/lib.rs
RUN cargo build --release 

RUN rm src/*.rs
COPY ./src/ ./src
RUN cargo build --release 

# Final image to contain the application, with chrome and chromedriver installed.
FROM base AS final

WORKDIR /app
COPY --from=build /app/target/release/basket-calendar .
COPY run.sh .

CMD ["/app/run.sh"]