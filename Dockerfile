FROM rust:latest

ENV RUST_BACKTRACE=full

WORKDIR /app

COPY ./ ./

RUN cargo build --release

CMD ["./target/release/epsilon_template"]