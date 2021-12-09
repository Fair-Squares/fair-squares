FROM rust:latest as builder

RUN apt update && apt install -y git clang curl libssl-dev llvm libudev-dev

WORKDIR /build

COPY . /build

RUN cargo build --release

FROM debian:buster-slim
LABEL org.opencontainers.image.source = "https://github.com/Fair-Squares/fair-squares"
COPY --from=builder /build/target/release/fs-node /usr/local/bin

RUN useradd -m -u 1000 -U -s /bin/sh -d /fs fs && \
	mkdir -p /fs/.local/share && \
	mkdir /data && \
	chown -R fs:fs /data && \
	ln -s /data /fs/.local/share/fs-node && \
	rm -rf /usr/bin /usr/sbin

USER fs
EXPOSE 30333 9933 9944
VOLUME ["/data"]

CMD ["/usr/local/bin/fs-node","--dev","--tmp"]