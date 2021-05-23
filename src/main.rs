extern crate pnet;
extern crate pnet_datalink;

use std::env;
use std::io::{self, Write};
use std::net::{AddrParseError, IpAddr, Ipv4Addr};
use std::process;

use pnet_datalink::{Channel, MacAddr, NetworkInterface};

use pnet::packet::arp::{ArpHardwareTypes, ArpOperations};
use pnet::packet::arp::{ArpPacket, MutableArpPacket};
use pnet::packet::ethernet::EtherTypes;
use pnet::packet::ethernet::MutableEthernetPacket;
use pnet::packet::{MutablePacket, Packet};

use tide::Request;
use tide::prelude::*;

fn get_mac_through_arp(target_ip: Ipv4Addr) -> MacAddr {
    let interfaces = pnet_datalink::interfaces();
    let interface = interfaces
        .into_iter()
        .find(|iface| iface.name == "br0")
        .unwrap();

    let source_ip = interface
        .ips
        .iter()
        .find(|ip| ip.is_ipv4())
        .map(|ip| match ip.ip() {
            IpAddr::V4(ip) => ip,
            _ => unreachable!(),
        })
        .unwrap();

    let (mut sender, mut receiver) = match pnet_datalink::channel(&interface, Default::default()) {
        Ok(Channel::Ethernet(tx, rx)) => (tx, rx),
        Ok(_) => panic!("Unknown channel type"),
        Err(e) => panic!("Error happened {}", e),
    };

    let mut ethernet_buffer = [0u8; 42];
    let mut ethernet_packet = MutableEthernetPacket::new(&mut ethernet_buffer).unwrap();

    ethernet_packet.set_destination(MacAddr::broadcast());
    ethernet_packet.set_source(interface.mac_address());
    ethernet_packet.set_ethertype(EtherTypes::Arp);

    let mut arp_buffer = [0u8; 28];
    let mut arp_packet = MutableArpPacket::new(&mut arp_buffer).unwrap();

    arp_packet.set_hardware_type(ArpHardwareTypes::Ethernet);
    arp_packet.set_protocol_type(EtherTypes::Ipv4);
    arp_packet.set_hw_addr_len(6);
    arp_packet.set_proto_addr_len(4);
    arp_packet.set_operation(ArpOperations::Request);
    arp_packet.set_sender_hw_addr(interface.mac_address());
    arp_packet.set_sender_proto_addr(source_ip);
    arp_packet.set_target_hw_addr(MacAddr::zero());
    arp_packet.set_target_proto_addr(target_ip);

    ethernet_packet.set_payload(arp_packet.packet_mut());

    sender
        .send_to(ethernet_packet.packet(), None)
        .unwrap()
        .unwrap();

    println!("Sent ARP request");

    let buf = receiver.next().unwrap();

    let arp = ArpPacket::new(&buf[MutableEthernetPacket::minimum_packet_size()..]).unwrap();

    println!("Received reply");

    arp.get_sender_hw_addr()
}

#[async_std::main]
async fn main() -> Result<(), std::io::Error> {
    
    tide::log::start();
    let mut app = tide::new();
    app.at("/").serve_file("static/index.html")?;
    app.at("/getid").get(|req: tide::Request<()>| async move {
        println!("ip: {:?}", req.remote().unwrap().parse::<Ipv4Addr>());
        let ip = req.remote().unwrap().split(':').next().unwrap().parse::<Ipv4Addr>().unwrap();
        let mac = get_mac_through_arp(ip);
        Ok(mac.to_string())
    });
    app.listen("suyemoto.com:80").await?;
    Ok(())
}
