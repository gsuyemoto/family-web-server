use pnet::datalink::{self, MacAddr, NetworkInterface};

use pnet::packet::ethernet::{EtherTypes, EthernetPacket, MutableEthernetPacket};
use pnet::packet::ip::{IpNextHeaderProtocol, IpNextHeaderProtocols};
use pnet::packet::ipv4::Ipv4Packet;
use pnet::packet::ipv6::Ipv6Packet;
use pnet::packet::Packet;

use std::env;
use std::io::{self, Write};
use std::net::IpAddr;
use std::process;

use log::{debug};

pub fn handle_ipv4_packet(ethernet: &EthernetPacket) -> Option<IpAddr> {
    match Ipv4Packet::new(ethernet.payload()) {
        Some(header) => Some(IpAddr::V4(header.get_destination())),
        None => { debug!("Malformed IPv4 Packet"); None },
    }
}

pub fn handle_ipv6_packet(ethernet: &EthernetPacket) -> Option<IpAddr> {
    match Ipv6Packet::new(ethernet.payload()) {
        Some(header) => Some(IpAddr::V6(header.get_destination())),
        None => { debug!("Malformed IPv6 Packet"); None },
    }
}

pub fn handle_ethernet_frame(e_packet: &EthernetPacket) -> (MacAddr, Option<IpAddr>) {
    let destination = match e_packet.get_ethertype() {
        EtherTypes::Ipv4 => handle_ipv4_packet(e_packet),
        EtherTypes::Ipv6 => handle_ipv6_packet(e_packet),
        _ => None,
    };

    (e_packet.get_source(), destination)
}
