FROM rust

ADD . /traceroute
WORKDIR /traceroute

ENTRYPOINT [ "/bin/bash" ]
