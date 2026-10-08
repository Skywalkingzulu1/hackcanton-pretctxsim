FROM rust:1.82-slim as builder

WORKDIR /app
COPY . .

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/pretxsim_poc /app/pretxsim_poc
COPY --from=builder /app/examples /app/examples

EXPOSE 8545

CMD ["./pretxsim_poc"]
