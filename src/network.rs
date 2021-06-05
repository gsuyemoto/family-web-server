use std::fs;
use std::collections::HashMap;

pub fn parse_leases() -> HashMap<String, String> {
    let leases = fs::read_to_string("/var/lib/dhcp/dhcpd.leases").unwrap();

    let ips = leases
        .split(|n| n == '\n')
        .filter(|n| n.contains("{"))
        .map(|n| n.get(6..16).unwrap().trim().to_string());

    let macs = leases
        .split(|n| n == '\n')
        .filter(|n| n.contains("hardware"))
        .map(|n| n.get(20..37).unwrap().to_string());

    ips
     .zip(macs)
     .into_iter()
     .collect::<HashMap<String, String>>()
}
