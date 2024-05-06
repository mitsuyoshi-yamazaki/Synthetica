FROM rust:1.67

WORKDIR /usr/src/synthetica
COPY ./synthetica .

RUN cargo install --path .