# traceroute
A simple implementation of the traceroute command in rust.

The program prints the path taken through the IP network from the source to the destination host, each line coresponding to a network gateway (physical device).

## Quick start

```sh
cargo run
```

```sh
$ traceroute google.com
traceroute to google.com (216.58.194.174), 64 hops max, 64 byte packets
  1 172.17.0.1 (172.17.0.1) 0.067 0.067 ms  0.555 ms  0.602 ms
  2 10.0.2.2 (10.0.2.2) 1.569 1.569 ms  0.535 ms  0.652 ms
  3 *  *  *
  4 192.168.0.2 (192.168.0.2) 1.342 1.342 ms  0.718 ms  1.057 ms
  5 nat.42.us.org (10.90.1.11) 0.704 0.704 ms  0.601 ms  0.494 ms
  6 64.62.224.30 (64.62.224.30) 1.148 1.148 ms  1.129 ms  5.674 ms
  7 64.62.224.253 (64.62.224.253) 4.028 4.028 ms  3.899 ms  4.293 ms
  8 64.62.224.249 (64.62.224.249) 4.374 4.374 ms  4.468 ms  4.204 ms
  9 v1851.core2.fmt2.he.net (216.218.200.77) 3.745 3.745 ms  3.748 ms  3.791 ms
 10 100ge4-1.core4.fmt2.he.net (184.104.192.253) 3.628 3.628 ms  3.825 ms  3.919 ms
 11 100ge14-1.core1.sjc2.he.net (184.105.213.157) 4.258 4.258 ms  4.638 ms  4.279 ms
 12 eqixsj-google-gige.google.com (206.223.116.21) 6.961 6.961 ms  4.526 ms  5.94 ms
 13 108.170.242.241 (108.170.242.241) 6.113 6.113 ms  6.294 ms  5.844 ms
 14 ??? (108.170.237.23) 5.296 5.296 ms  4.794 ms  4.909 ms
 15 sfo07s13-in-f14.1e100.net (216.58.194.174) 4.709 4.709 ms  5.062 ms  5.178 ms
```

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

The command is implemented by repeatedly sending IMCP ECHO packets to the destination with ttl values increasing from 1 to 64. Ttl values are decremented by 1 at each gateway, and packets return as ICMP TIME_EXCEEDED packets when ttl reaches 0. So by starting with ttl=1, we can obtain the IP address and round trip time of all devices on the path to the host.
