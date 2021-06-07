use std::fs;
use std::collections::HashMap;

pub fn get_mac_from_ip(ip: &str) -> Option<String> {
    let leases = fs::read_to_string("/var/lib/dhcp/dhcpd.leases").unwrap();
    
    let ips: Vec<String> = leases
        .split(|n| n == '}')
        .filter(|n| n.contains("hardware"))
        .collect::<String>()
        .split(|n| n == '\n')
        .filter(|n| n.contains("{"))
        .map(|n| n.get(6..16).unwrap().trim().to_string())
        .collect();
    
    let macs: Vec<String> = leases
        .split(|n| n == '\n')
        .filter(|n| n.contains("hardware"))
        .map(|n| n.get(20..37).unwrap().to_string())
        .collect();
    
    let macs_ips = macs
                    .iter()
                    .zip(ips.iter())
                    .collect::<HashMap<&String, &String>>();
    
    let mut ips_macs = HashMap::new();
    
    for (key, value) in macs_ips {
        ips_macs.insert((*value).clone(), (*key).clone());
    }

    match ips_macs.get(ip) {
        Some(mac) => Some(mac.clone()),
        None => None,
    }
}
