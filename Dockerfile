FROM rust:1.34 AS builder
WORKDIR /rpb/build

COPY . .
RUN cargo build --release


FROM debian
WORKDIR /rpb/bin

COPY --from=builder /rpb/build/target/release/presence-ws-server .

CMD /rpb/bin/presence-ws-server
