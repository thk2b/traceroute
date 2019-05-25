extern crate pnet;
extern crate libc;
extern crate rand;
extern crate dns_lookup;

use std::io::{self, Write};
use std::time::{Duration, Instant};
use std::net::{IpAddr, Ipv4Addr, ToSocketAddrs};

use std::mem;
use pnet::{
    util::checksum,
    packet::{Packet, MutablePacket,
        icmp::{
            self, IcmpTypes, echo_request::{MutableEchoRequestPacket},
            echo_reply::{EchoReplyPacket},
        },
        ip::IpNextHeaderProtocols,
    },
    transport::{
        transport_channel,
        TransportChannelType::Layer4,
        TransportProtocol::Ipv4,
        icmp_packet_iter,
        TransportSender,
        IcmpTransportChannelIterator,
    },
};
use rand::Rng;

static MAX_TTL: u8 = 64;
static TIMEOUT: u64 = 3;

fn new_echo_packet(buf: &mut [u8], ttl: u8) -> MutableEchoRequestPacket {
    let mut p = MutableEchoRequestPacket::new(buf).expect("echo packet");
    p.set_icmp_type(icmp::IcmpTypes::EchoRequest);
    p.set_sequence_number(ttl.into());
    let id = rand::thread_rng().gen_range(0, std::u16::MAX);
    p.set_identifier(id);
    p.set_payload(&[0u8; 16-8]);
    let cksum = checksum(&p.packet_mut(), 1);
    p.set_checksum(cksum);
    p
}

fn time_diff(sent_at: Instant) -> f32 {
    let diff = Instant::now().duration_since(sent_at);
    diff.as_micros() as f32 / 1000.0
}

struct Pinger<'a> {
    tx: TransportSender,
    rx: IcmpTransportChannelIterator<'a>,
}

impl<'a> Pinger<'a> {
    fn new(tx: TransportSender, rx: IcmpTransportChannelIterator<'a>) -> Pinger {
        Pinger { tx, rx }
    }

    fn ping(&mut self, dst: &Ipv4Addr, ttl: u8) -> Result<(f32, IpAddr), io::Error> {
        let mut req_buf = [0u8; 64];
        let packet = new_echo_packet(&mut req_buf, ttl);
        let packet = packet.to_immutable();
        let id = packet.get_identifier();
        self.tx.set_ttl(ttl).expect("set ttl");
        let sent_at = Instant::now();
        if let Err(e) = self.tx.send_to(packet, IpAddr::V4(*dst)) {
            return Err(e)
        }
        loop { // loop until timeout or we recieved the packet
            match self.rx.next() {
            // match self.rx.next_with_timeout(3000) {
                Ok((res, addr)) => {
                    match res.get_icmp_type() {
                        IcmpTypes::TimeExceeded => (),
                        IcmpTypes::EchoReply => {
                            let res = EchoReplyPacket::new(res.packet()).expect("echo packet");
                            if res.get_identifier() != id {// TODO: validate cksum
                                continue
                            }
                        }
                        _ => continue
                    }
                    return Ok((time_diff(sent_at), addr))
                },
                Err(e) => return Err(e)
            }
        }
    }
}

fn duration_to_timeval(dur: Duration) -> libc::timeval {
    libc::timeval {
        tv_sec: dur.as_secs() as libc::time_t,
        tv_usec: dur.subsec_micros() as libc::c_long,
    }
}

fn set_socket_receive_timeout(socket: libc::c_int, t: Duration) -> io::Result<()> {
    let ts = duration_to_timeval(t);
    let r = unsafe {
        libc::setsockopt(socket, libc::SOL_SOCKET, libc::SO_RCVTIMEO,
                   (&ts as *const libc::timeval) as *const libc::c_void,
                   mem::size_of::<libc::timeval>() as libc::socklen_t
        )
    };
    if r < 0 {
        Err(io::Error::last_os_error()) 
    } else if r > 0 {
        Err(io::Error::new(io::ErrorKind::Other, format!("Unknown return value from getsockopt(): {}", r)))
    } else {
        Ok(())
    }
}

fn traceroute(dst: Ipv4Addr, hostname: &str) -> Result<(), io::Error> {
    let protocol = Layer4(Ipv4(IpNextHeaderProtocols::Icmp));
    let (tx, mut rx) = transport_channel(1024, protocol)?;
    set_socket_receive_timeout(rx.socket.fd, Duration::from_secs(TIMEOUT))?;
    let rx = icmp_packet_iter(&mut rx);
    let mut pinger = Pinger::new(tx, rx);

    println!("traceroute to {:} ({:}), {:} hops max, {:} byte packets",
        hostname, dst, MAX_TTL, 64
    );
    for ttl in 1..MAX_TTL {
        for i in 0..3 {
            match pinger.ping(&dst, ttl) {
                Ok((time, ip)) => {
                    if i == 0 {
                        let hostname = if let Ok(name) = dns_lookup::lookup_addr(&ip) {
                            name
                        } else {
                            "???".to_string()
                        };
                        print!(" {:2} {:} ({:}) {:}", ttl, hostname, ip, time);
                    };
                    print!(" {:} ms ", time);
                    if i == 2 && ip == dst { // reached destination and did 3 rounds
                        println!(" ");
                        return Ok(())
                    }
                },
                Err(e) => {
                    match e.kind() {
                        io::ErrorKind::WouldBlock => print!(" * "),
                        _ => println!("error {:}", e),
                    };
                }
            }
            io::stdout().flush();
        }
        println!("");
    }
    Ok(())
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        return println!("usage: ./traceroute host");
    }
    let mut dst;
    match (args[1].as_str(), 0).to_socket_addrs() {
        Ok(mut iter) => {
            dst = iter.next().expect("empty dns record");
        },
        Err(e) => {
            return println!("{:}", e);
        }
    };
    if let IpAddr::V4(ip) = dst.ip() {
        if let Err(e) = traceroute(ip, &args[1]) {
            println!("{:}", e);
        }
    } else {
        println!("ipv6 is not suported");
    };
}
