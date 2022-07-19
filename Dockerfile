FROM docker.io/paritytech/ci-linux:staging as builder

WORKDIR /substrate
COPY . /substrate
RUN cargo build --release --locked

FROM docker.io/library/ubuntu:20.04
LABEL org.opencontainers.image.source = "https://github.com/Fair-Squares/fair-squares"

COPY --from=builder /substrate/target/release/fs-node /usr/local/bin

RUN useradd -m -u 1000 -U -s /bin/sh -d /fs fs && \
	mkdir -p /data /fs/.local/share && \
	chown -R fs:fs /data && \
	ln -s /data /fs/.local/share/fs-node && \
	rm -rf /usr/bin /usr/sbin \
# check if executable works in this container
	/usr/local/bin/fs-node --version

USER fs
EXPOSE 30333 9933 9944 9615
VOLUME ["/data"]

CMD ["/usr/local/bin/fs-node"]