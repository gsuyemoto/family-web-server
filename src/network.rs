use pnet::datalink::{self, MacAddr, Channel, NetworkInterface};
use pnet::packet::arp::{ArpHardwareTypes, ArpOperations};
use pnet::packet::arp::{ArpPacket, MutableArpPacket};
use pnet::packet::ethernet::{EtherTypes, MutableEthernetPacket, EthernetPacket};
use pnet::packet::{MutablePacket, Packet};
use pnet::packet::udp::UdpPacket;
use pnet::transport;

use std::net::{AddrParseError, IpAddr, Ipv4Addr};
use log::{error, debug, info};
use crate::parsepacket;

pub fn sniff() {
    let interfaces = datalink::interfaces();
    let interface = interfaces
        .into_iter()
        .find(|iface| iface.name == "br0")
        .unwrap();
    
    let (_, mut receiver) = match datalink::channel(&interface, Default::default()) {
        Ok(Channel::Ethernet(tx, rx)) => (tx, rx),
        Ok(_) => panic!("Unknown channel type"),
        Err(e) => panic!("Error happened {}", e),
    };

    // get 10 packets from ethernet layer
    for index in (1..=30) {
        match receiver.next() {
            Ok(packet) => {
                let enet_packet = EthernetPacket::new(packet).unwrap();
                let (mac, ip)   = parsepacket::handle_ethernet_frame(&enet_packet);
                debug!("sniffed {}: {}", index, mac);

                // if mac == "28:37:37:0B:19:BB".parse::<MacAddr>().unwrap() {
                if mac == "80:B0:3D:05:39:EF".parse::<MacAddr>().unwrap() {
                    info!("Found iPhone! Destination: {:?}", ip);
                }
            },
            Err(e) => {
                error!("Error while reading packets from br0");
            },
        }
    }
}

pub fn get_mac_through_arp(target_ip: Ipv4Addr) -> MacAddr {
    let interfaces = datalink::interfaces();
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

    let (mut sender, mut receiver) = match datalink::channel(&interface, Default::default()) {
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
