FROM rust:1.74.1

# Set the RUST_LOG environment variable
ENV RUST_LOG=info

COPY ./ ./

RUN cargo build --release

EXPOSE 3000

CMD ["./target/release/rust-s3-scylla"]
