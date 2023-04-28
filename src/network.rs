extern crate iptables;

use std::fs;

pub fn get_addr(ip: Option<&str>, mac: Option<&str>) -> Option<String> {
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

    // Provide IP if you want a MAC
    // Provide MAC if you want the IP
    match ip {
        Some(ip) => {
            for (index, ip_found) in ips.iter().enumerate() {
                if ip_found.eq(&ip) {
                    return Some(macs[index].clone())
                }
            }
        },
        None => {
            let mac = mac.unwrap();
            for (index, mac_found) in macs.iter().enumerate() {
                if mac_found.eq(&mac) {
                    return Some(macs[index].clone())
                }
            }
        },
    }

    None
}

pub fn block_ip(ip: &str) {
    let ipt     = iptables::new(false).unwrap();

    let rule1   = format!("-d {} -j DROP", ip);
    let rule2   = format!("-s {} -j DROP", ip);
    let exists  = ipt.exists("filter", "FORWARD", &rule1).unwrap();
    
    if !exists {
        ipt.insert("filter", "FORWARD", &rule1, 1).expect("Unable to set filter INPUT 1");
        ipt.insert("filter", "FORWARD", &rule2, 2).expect("Unable to set filter INPUT 2");
    }
}

pub fn unblock_ip(ip: &str) {
    let ipt = iptables::new(false).unwrap();

    let rule1 = format!("-d {} -j DROP", ip);
    let rule2 = format!("-s {} -j DROP", ip);

    ipt.delete("filter", "FORWARD", &rule1).expect("Unable to delete filter INPUT 1");
    ipt.delete("filter", "FORWARD", &rule2).expect("Unable to delete filter INPUT 2");
}
