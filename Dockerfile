FROM rust

WORKDIR /usr/src/services
COPY . .

RUN cargo install diesel_cli --no-default-features --features postgres
RUN cargo build --release
CMD diesel migration run && /usr/src/services/target/release/werewolf_services
