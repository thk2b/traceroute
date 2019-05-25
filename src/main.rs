extern crate pnet;

use std::error::Error;
use std::io;
use std::net::{IpAddr, Ipv4Addr};
use pnet::{
    util::checksum,
    packet::{
        MutablePacket,
        icmp::{
            self,
            echo_request::{
                MutableEchoRequestPacket,
                EchoRequestPacket,
            },
            echo_reply::EchoReplyPacket
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

static DEFAULT_TTL: u16 = 64;

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

fn new_echo_packet(buf: &mut [u8], ttl: u16) -> MutableEchoRequestPacket {
    let mut p = MutableEchoRequestPacket::new(buf).expect("echo packet");
    p.set_icmp_type(icmp::IcmpTypes::EchoRequest);
    p.set_sequence_number(ttl);
    p.set_identifier(1); // TODO
    p.set_payload(&[1u8; 16-8]); //TODO
    let cksum = checksum(&p.packet_mut(), 1);
    p.set_checksum(cksum);
    p
}

fn echo_packets_match(req: &EchoRequestPacket, res: &EchoReplyPacket) -> bool {
    true //TODO
}

fn echo_packets_time_diff(req: &EchoRequestPacket, res: &EchoReplyPacket) -> f32 {
    0.123 // TODO
}

struct Pinger<'a> {
    tx: TransportSender,
    rx: IcmpTransportChannelIterator<'a>,
}

impl<'a> Pinger<'a> {
    fn new(tx: TransportSender, rx: IcmpTransportChannelIterator<'a>) -> Pinger {
        Pinger { tx, rx }
    }

    fn ping(&mut self, dst: &Ipv4Addr, buf: &mut [u8], ttl: u16) -> Result<f32, io::Error> {
        let packet = new_echo_packet(buf, ttl);
        //set ttl on tx.socket
        match self.tx.send_to(packet, IpAddr::V4(*dst)) {
            Ok(sent) => println!("sent {:}", sent),
            Err(e) => {
                println!("error sending {:}", e);
                return Err(e)
            },
        };
        loop { // loop until timeout or we recieved the packet
            match self.rx.next() {
                Ok(res) => {
                    println!("recieved {:?}", &res);
                    // if echo_packets_match(&packet, &res) {
                        // return Ok(echo_packets_time_diff(&packet, &res))//res))
                    // }
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
    let (mut tx, mut rx) = transport_channel(1024, protocol)?;//TODO: timeout
    let mut rx = icmp_packet_iter(&mut rx);
    let mut pinger = Pinger::new(tx, rx);
    let mut buf = [0u8; 64];

    for ttl in 1..DEFAULT_TTL { // do this 3 times
        for _ in 0..3 {
            match pinger.ping(dst, &mut buf[..], ttl) {
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
