use std::fs;

pub fn get_mac_from_ip(ip: &str) -> Option<String> {
    let leases = fs::read_to_string("/var/lib/dhcp/dhcpd.leases").unwrap();
    
    let ips: Vec<String> = leases
        .split(|n| n == '}')
        .filter(|n| n.contains("hardware"))
        .collect::<String>()
        .split(|n| n == '\n')
        .filter(|n| n.contains("{"))
        .map(|n| n.strip_suffix("{").unwrap())
        .map(|n| n.get(6..).unwrap().trim().to_string())
        .collect();

    let macs: Vec<String> = leases
        .split(|n| n == '\n')
        .filter(|n| n.contains("hardware"))
        .map(|n| n.get(20..37).unwrap().to_string())
        .collect();

    for (index, ip_found) in ips.iter().enumerate() {
        if ip_found.eq(&ip) {
            return Some(macs[index].clone())
        }
    }

    None
}
