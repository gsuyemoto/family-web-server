use crate::{Pool};
use crate::schema::{devices, users};
use crate::network;

use smoltcp::phy::wait as phy_wait;
use smoltcp::phy::{Device, RxToken, RawSocket};
use smoltcp::wire::{EthernetAddress, PrettyPrinter, EthernetFrame, TcpPacket, Ipv4Packet, IpProtocol, EthernetProtocol};

use tokio::task;
use tokio::time::{self, interval, Instant, Duration};

use std::os::unix::io::AsRawFd;
use diesel::prelude::*;
use log::{error, debug};

struct DeviceToTrack {
    user_id:            i32,
    mac:                EthernetAddress,
    ip:                 String, 
    last_check:         u64,
    last_last_check:    u64,
    is_watching:        bool,
}

pub async fn begin_tracking(db: Pool) {
    
    const CHECK_STREAMING: u64          = 30;
    const CHECK_FOR_NEW_DEVICE: u64     = 60 * 60; // check for new devices every hr
    const STREAMING_THRESHOLD: u64      = 1_000_000;
    let mut socket                      = RawSocket::new("br0".as_ref()).unwrap();
    let mut all_devices: Vec<DeviceToTrack>     = Vec::new();
    let mut interval                            = interval(Duration::from_secs(CHECK_STREAMING));
    let mut check_new_devices                   = Instant::now();
    let mut check_streaming                     = Instant::now();
    let mut devices_not_init                    = true;
    let conn                                    = db
                                                    .get()
                                                    .expect("couldn't get db connection from pool");
    loop {
        let elapsed_new_device = check_new_devices.elapsed().as_secs();
        if devices_not_init || elapsed_new_device > CHECK_FOR_NEW_DEVICE {
                
            let devices_from_db = 
                devices::table
                .filter(devices::is_tracked.eq(1))
                .select((devices::user_id, devices::addr_mac, devices::addr_ip))
                .load::<(i32, String, String)>(&conn)
                    .map_err(|err| error!("Problem getting devices to track: {}", err))
                    .unwrap();

            all_devices.clear();
            for dev in devices_from_db {
                all_devices.push( 
                    DeviceToTrack {
                    user_id:            dev.0,
                    mac:                dev.1.parse::<EthernetAddress>().unwrap(),
                    ip:                 dev.2,
                    last_check:         0,
                    last_last_check:    0,
                    is_watching:        false,
                });
            }

            // reset time to check for new devices
            check_new_devices  = Instant::now();
            devices_not_init   = false;
        }

        // ONLY TRACK DEVICES IF THERE ARE DEVICES TO TRACK
        if all_devices.len() > 0 {
            phy_wait(socket.as_raw_fd(), None).unwrap();
            let (rx_token, _) = socket.receive().unwrap();

            rx_token.consume(smoltcp::time::Instant::now(), |buffer| {

                let frame = EthernetFrame::new_unchecked(&buffer);
                if frame.ethertype() == EthernetProtocol::Ipv4 
                {
                    for device in &mut all_devices {
                        if frame.dst_addr() == device.mac || frame.src_addr() == device.mac 
                        {
                            if let Ok(ipv4) = Ipv4Packet::new_checked(frame.payload()) {
                                device.last_check += ipv4.total_len() as u64;
                            }
                        }
                    }
                }
    
                let elapsed_streaming = check_streaming.elapsed().as_secs();
                if elapsed_streaming > CHECK_STREAMING 
                {
                    for device in &mut all_devices {
                        if device.last_check > STREAMING_THRESHOLD &&
                            device.last_last_check > STREAMING_THRESHOLD
                        {
                            device.is_watching = true;
                            debug!("{} --->  {}", device.mac, device.is_watching);

                            let updated_row = diesel::update(
                                devices::table.filter(
                                    devices::addr_mac.eq(device.mac.to_string())))
                                    .set(devices::is_watching.eq(1))
                                    .execute(&conn);

                            let updated_row = diesel::update(
                                users::table.filter(
                                    users::user_id.eq(device.user_id)))
                                    .set(users::points.eq(users::points - 1))
                                    .execute(&conn);

                            // CHECK IF USER HAS REACHED 0 POINTS!!
                            // IF SO, BLOCK ALL DEVICES
                            let points = users::table.filter(
                                users::user_id.eq(device.user_id))
                                .select(users::points)
                                .execute(&conn);

                            if points.unwrap() == 0 {
                                debug!("Blocking device ip: {}", device.ip);
                                network::block_ip(device.ip.clone());
                            }
                        }
                        else
                        {
                            let updated_row = diesel::update(
                                devices::table.filter(
                                    devices::addr_mac.eq(device.mac.to_string())))
                                    .set(devices::is_watching.eq(0))
                                    .execute(&conn);

                            debug!("Not watchin: {:?}", updated_row);

                            device.is_watching = false;
                        }
    
                        device.last_last_check      = device.last_check;
                        device.last_check           = 0;
                        check_streaming        = Instant::now();
                    }
                }
    
                Ok(())
            }).unwrap();
        }
    }
}

#[cfg(windows)]
pub async fn begin_tracking(db: Pool) {
    const CHECK_STREAMING: u64                  = 30;
    const CHECK_FOR_NEW_DEVICE: u64             = 60 * 60; // check for new devices every hr
    const STREAMING_THRESHOLD: u64              = 1_000_000;
         
    let mut all_devices: Vec<DeviceToTrack>     = Vec::new();
    let mut interval                            = interval(Duration::from_secs(CHECK_STREAMING));
    let mut check_new_devices                   = Instant::now();
    let mut devices_not_init                    = true;
    let conn                                    = db
                                                    .get()
                                                    .expect("couldn't get db connection from pool");
    loop {
        interval.tick().await;

        if devices_not_init || check_new_devices.elapsed().as_secs() > CHECK_FOR_NEW_DEVICE {

            let devices_from_db = 
                devices::table
                .filter(devices::is_tracked.eq(1))
                .select((devices::user_id, devices::addr_mac, devices::addr_ip))
                .load::<(i32, String, String)>(&conn)
                    .map_err(|err| error!("Problem getting devices to track: {}", err))
                    .unwrap();

            all_devices.clear();
            for dev in devices_from_db {
                all_devices.push( 
                    DeviceToTrack {
                    user_id:            dev.0,
                    mac:                dev.1.parse::<EthernetAddress>().unwrap(),
                    ip:                 dev.2,
                    last_check:         0,
                    last_last_check:    0,
                    is_watching:        false,
                });
            }

            // reset time to check for new devices
            check_new_devices  = Instant::now();
            devices_not_init   = false;
        }

        // ONLY TRACK DEVICES IF THERE ARE DEVICES TO TRACK
        if all_devices.len() > 0 {
            let mut socket = RawSocket::new("br0".as_ref()).unwrap();
            phy_wait(socket.as_raw_fd(), None).unwrap();
            let (rx_token, _) = socket.receive().unwrap();

            rx_token.consume(smoltcp::time::Instant::now(), |buffer| {

                let frame = EthernetFrame::new_unchecked(&buffer);
                if frame.ethertype() == EthernetProtocol::Ipv4 
                {
                    for device in &mut all_devices {
                        if frame.dst_addr() == device.mac || frame.src_addr() == device.mac 
                        {
                            if let Ok(ipv4) = Ipv4Packet::new_checked(frame.payload()) {
                                device.last_check += ipv4.total_len() as u64;
                            }
                        }
                    }
                }

                for device in &mut all_devices {
                    if device.last_check > STREAMING_THRESHOLD &&
                        device.last_last_check > STREAMING_THRESHOLD
                    {
                        device.is_watching = true;
                        debug!("{} --->  {}", device.mac, device.is_watching);

                        let updated_row = diesel::update(
                            devices::table.filter(
                                devices::addr_mac.eq(device.mac.to_string())))
                                .set(devices::is_watching.eq(1))
                                .execute(&conn);

                        let updated_row = diesel::update(
                            users::table.filter(
                                users::user_id.eq(device.user_id)))
                                .set(users::points.eq(users::points - 1))
                                .execute(&conn);

                        // CHECK IF USER HAS REACHED 0 POINTS!!
                        // IF SO, BLOCK ALL DEVICES
                        let points = users::table.filter(
                            users::user_id.eq(device.user_id))
                            .select(users::points)
                            .execute(&conn);

                        if points.unwrap() == 0 {
                            debug!("Blocking device ip: {}", device.ip);
                            network::block_ip(device.ip.clone());
                        }
                    }
                    else
                    {
                        let updated_row = diesel::update(
                            devices::table.filter(
                                devices::addr_mac.eq(device.mac.to_string())))
                                .set(devices::is_watching.eq(0))
                                .execute(&conn);

                        debug!("{} Not watching: {}", device.ip, device.last_check);

                        device.is_watching = false;
                    }

                    device.last_last_check      = device.last_check;
                    device.last_check           = 0;
                }

                Ok(())
            }).unwrap();
        }
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn a_test() {
    }
}
