# traceroute
An implementation of the traceroute command in rust

## Quick start

You must be root to run the command. A dockerfile is provided to run inside a Debian VM:

```sh
docker-machine create --driver virtualbox traceroute
docker-machine start traceroute
eval $(docker-machine env traceroute)
docker build -t local/traceroute .
docker run -itv `pwd`:/ping local/traceroute
```

## Usage

`usage: ./traceroute host`

## Implementation

