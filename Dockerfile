FROM rust:1.34 AS builder
WORKDIR /rpb/build

# Pre-compile the dependencies
RUN USER=rpb cargo init --name=precompile-deps
COPY Cargo.toml Cargo.lock ./
RUN cargo build --release

# Compile the actual server
COPY . .
RUN touch src/main.rs
RUN cargo build --release




FROM debian
WORKDIR /rpb/bin
COPY --from=builder /rpb/build/target/release/presence-ws-server .
CMD /rpb/bin/presence-ws-server
