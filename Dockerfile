FROM rust

ADD . /traceroute
WORKDIR /traceroute

# RUN cargo build || true

ENTRYPOINT [ "/bin/bash" ]
