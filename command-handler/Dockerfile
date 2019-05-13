FROM rust:1.34.2-slim-stretch AS builder
RUN apt-get update
RUN apt-get install -y build-essential
RUN apt-get install -y curl
RUN apt-get install -y libssl-dev
RUN apt-get install -y gcc-multilib
RUN apt-get install -y libpq-dev
RUN apt-get install -y pkg-config
RUN apt-get install -y llvm-3.9-dev libclang-3.9-dev clang-3.9
COPY . .
RUN cargo build --release

FROM debian:stretch-slim
RUN apt-get update && apt-get install -y curl && apt-get install -y libpq-dev

COPY --from=builder /target/release/rust-command-handler .
CMD ["./rust-command-handler"]