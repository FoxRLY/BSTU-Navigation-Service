FROM rust:latest as build
RUN USER=root cargo new actix_navigation_service
WORKDIR /actix_navigation_service
RUN echo $(pwd)
RUN echo $(ls)
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml
RUN cargo build --release
RUN rm src/*.rs
COPY ./src ./src
RUN rm ./target/release/actix_navigation_service*
RUN cargo build --release

FROM rust:latest
COPY --from=build /actix_navigation_service/target/release/actix_navigation_service .
COPY ./images.json .
COPY ./classrooms.json .
CMD ["./actix_navigation_service"]
