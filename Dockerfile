FROM rustlang/rust:nightly

COPY ./ ./

RUN rustc --version
RUN cargo +nightly build --release

CMD ["./target/release/epsilon_template"]