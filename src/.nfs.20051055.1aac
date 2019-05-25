extern crate pnet;

use std::error::Error;
use std::io;
use std::time::Instant;
use std::net::{IpAddr, Ipv4Addr};
use std::slice;
use std::mem;
use pnet::{
    util::checksum,
    packet::{
        Packet,
        MutablePacket,
        icmp::{
            self,
            IcmpTypes,
            Icmp,
            IcmpPacket,
            echo_request::{
                MutableEchoRequestPacket,
                EchoRequestPacket,
            },
            echo_reply::{
                EchoReplyPacket,
                EchoReply,
            }
        },
        ip::IpNextHeaderProtocols,
    },
    transport::{
        transport_channel,
        TransportChannelType::Layer4,
        TransportProtocol::Ipv4,
        icmp_packet_iter,
        TransportSender,
        TransportReceiver,
        IcmpTransportChannelIterator,
    },
};

static DEFAULT_TTL: u8 = 64;

// struct EchoPacket { }

// impl EchoPacket {
//     fn new(buf: &mut [u8], ttl: u16) {
//         let mut p = MutableEchoRequestPacket::new(buf).expect("echo packet");
//         p.set_icmp_type(icmp::IcmpTypes::EchoRequest);
//         p.set_sequence_number(ttl);
//         p.set_identifier(1); // TODO
//         p.set_payload(&[1u8; 16-8]); //TODO
//         let cksum = checksum(&p.packet_mut(), 1);
//         p.set_checksum(cksum);
//         p
//     }

// }

fn new_echo_packet(buf: &mut [u8], ttl: u8) -> MutableEchoRequestPacket {
    let mut p = MutableEchoRequestPacket::new(buf).expect("echo packet");
    p.set_icmp_type(icmp::IcmpTypes::EchoRequest);
    p.set_sequence_number(ttl.into());
    p.set_identifier(1); // TODO
    let now = Instant::now();
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

    fn ping(&mut self, dst: &Ipv4Addr, ttl: u8) -> Result<f32, io::Error> {
        let mut req_buf = [0u8; 64];
        let mut res_buf = [0u8; 64];

        let packet = new_echo_packet(&mut req_buf, ttl);
        let packet = packet.to_immutable();
        let id = packet.get_identifier();
        self.tx.set_ttl(ttl).expect("set ttl");
        let sent_at = Instant::now();
        match self.tx.send_to(packet, IpAddr::V4(*dst)) {
            Ok(sent) => println!("sent {:} bytes with ttl={:}", sent, ttl),
            Err(e) => {
                println!("error sending {:}", e);
                return Err(e)
            },
        };
        loop { // loop until timeout or we recieved the packet
            match self.rx.next() {
                Ok((res, addr)) => {
                    println!("recieved {:?} from {:}", &res, addr);
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
                    return Ok(time_diff(sent_at))
                },
                Err(e) => {
                    println!("error recieving {:}", e);
                    return Err(e)
                },
            }
        }
    }
}

fn traceroute(dst: &Ipv4Addr) -> Result<(), io::Error> {
    let protocol = Layer4(Ipv4(IpNextHeaderProtocols::Icmp));
    let (tx, mut rx) = transport_channel(1024, protocol)?; // TODO: timeout
    let rx = icmp_packet_iter(&mut rx);
    let mut pinger = Pinger::new(tx, rx);

    // for ttl in 1..DEFAULT_TTL { // do this 3 times
    for ttl in 1..65 { // do this 3 times
        for _ in 0..3 {
            match pinger.ping(dst, ttl) {
                Ok(time) => print!("{:} ms", time),
                Err(e) => println!("error {:}", e),
            }
            println!("");
        }
    }
    Ok(())
}

fn main() -> Result<(), io::Error> {
    let dst = Ipv4Addr::new(157,240,22,35); //TODO: resolve host, FQHN, ip
    traceroute(&dst)

    // let mut rx = icmp_packet_iter(&mut rx);

    // let mut icmpbuf = vec![0u8; 16];
    // let mut p = MutableEchoRequestPacket::new(&mut icmpbuf[..]).expect("echo packet");
    // p.set_icmp_type(icmp::IcmpTypes::EchoRequest);
    // p.set_sequence_number(1);
    // p.set_identifier(1);
    // p.set_payload(&[1u8; 16-8]);
    // let cksum = checksum(&p.packet_mut(), 1);
    // p.set_checksum(cksum);

    // println!("sending {:?}", &p);
    // tx.send_to(p, IpAddr::V4(dst)).expect("send");
    // loop {
    //     let packet = rx.next().expect("next packet");
    //     println!("recieved {:?}", packet);
    // };
}
