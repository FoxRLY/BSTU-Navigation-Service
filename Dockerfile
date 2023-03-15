FROM debian:buster-slim

RUN mkdir /app
COPY ./ /app
CMD ["/app/release/actix_navigation_service"]
