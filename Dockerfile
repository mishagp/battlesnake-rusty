FROM rust:1.87

EXPOSE 8000/tcp

COPY . /usr/app
WORKDIR /usr/app

RUN cargo install --path .

CMD ["battlesnake-rusty"]
