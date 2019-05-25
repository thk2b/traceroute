extern crate pnet;

use std::net::{IpAddr, Ipv4Addr};
use pnet::{
    util::checksum,
    packet::{
        MutablePacket,
        icmp::{
            self, IcmpPacket,
            echo_request::{
                MutableEchoRequestPacket, EchoRequestPacket
            },
        },
        ip::{
            self,
            IpNextHeaderProtocols,
        },
        ipv4::MutableIpv4Packet
    },
    transport::{
        TransportReceiver,
        transport_channel,
        TransportChannelType::Layer3,
        icmp_packet_iter,
    },
};

// fn newEchoReq(ipbuf: &[u8], icmpbuf: &[u8]) -> EchoRequestPacket {
//     let ip = MutableIpv4Packet::new(ipbuf);
// }

static IPV4_HEADER_LEN: usize = 21;
static ICMP_HEADER_LEN: usize = 8;
static ICMP_PAYLOAD_LEN: usize = 26;

fn main() {
    let (mut tx, mut rx) = transport_channel(1024, Layer3(IpNextHeaderProtocols::Icmp))
        .expect("create transport channel");
    let dst = Ipv4Addr::new(157,240,22,35);
    let mut rx = icmp_packet_iter(&mut rx);

    let mut ipbuf = [0u8; 21+8+26];
    let mut ip = MutableIpv4Packet::new(&mut ipbuf).expect("ip packet");
    ip.set_version(4);
    ip.set_header_length(IPV4_HEADER_LEN.to_be() as u8);
    ip.set_total_length((IPV4_HEADER_LEN + ICMP_HEADER_LEN + ICMP_PAYLOAD_LEN) as u16);
    ip.set_ttl(64);
    ip.set_next_level_protocol(IpNextHeaderProtocols::Icmp);
    ip.set_destination(dst);

    let mut icmpbuf = [0u8; 8+26];
    let mut p = MutableEchoRequestPacket::new(&mut icmpbuf).expect("echo packet");
    p.set_icmp_type(icmp::IcmpTypes::EchoRequest);
    p.set_sequence_number(1);
    p.set_identifier(1);
    let cksum = checksum(&p.packet_mut(), 2);
    p.set_checksum(cksum);
    p.set_payload(&[1u8; 26]);

    let cksum = checksum(&ip.packet_mut(), 2);
    ip.set_checksum(cksum);
    ip.set_payload(p.packet_mut());
    println!("{:?}", ip);
    println!("{:?}", p);

    let sent = tx.send_to(ip, IpAddr::V4(dst)).expect("send");
    println!("sent: {:?}", sent);
    loop {
        let packet = rx.next().expect("next packet");
        println!("{:?}", packet);
    };
}
