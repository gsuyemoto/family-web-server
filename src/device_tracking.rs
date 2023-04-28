use crate::{models, network, Pool};
use crate::schema::{devices, users};

use smoltcp::phy::wait as phy_wait;
use smoltcp::phy::{Device, RxToken, RawSocket};
use smoltcp::wire::{EthernetAddress, EthernetFrame, Ipv4Packet, EthernetProtocol};
use smoltcp::Error;

use tokio::time::{self, Instant, Duration};
use tokio::sync::Notify;

use std::sync::{Arc};
use std::os::unix::io::AsRawFd;
use diesel::prelude::*;
use log::{error, debug};

pub async fn begin_tracking(db: Pool, rcv: Arc<Notify>) {
    
    const CHECK_THROUGHPUT: u64         = 10;
    const CHECK_NEW_DEVICE: u64         = 60 * 5; // check for new devices every 5min
    const THROUGHPUT_THRESHOLD: i32     = 9_000;

    let all_devices: &mut Vec<models::Device>   = &mut Vec::new();
    let mut check_new_device                    = Instant::now();

    let conn                                    = db
                                                    .get()
                                                    .expect("couldn't get db connection from pool");
    loop {
        // loop every 1 secs
        let mut interval = time::interval(Duration::from_secs(1));
        interval.tick().await;

        if all_devices.is_empty()|| check_new_device.elapsed().as_secs() > CHECK_NEW_DEVICE {
            let devices_from_db = 
                devices::table
                .filter(devices::is_tracked.eq(1))
                .load(&conn);

            match devices_from_db {
                Ok(devs)  => {
                    for d in devs{
                        all_devices.push(d);
                    }
                },
                Err(e)    => {
                    error!("Problem getting devices to track: {}", e);
                    continue;
                },
            }

            // reset time to check for new devices
            check_new_device  = Instant::now();
        }
            
        if all_devices.is_empty() { 
            debug!("No devices to track. Waiting for more to be added.");
            rcv.notified().await;
            debug!("Received notification more devices were added.");
            continue;
        }
        
        let mut check_throughput    = Instant::now();
        let mut socket = RawSocket::new("br0"
                        .as_ref())
                        .unwrap();

        loop {
            phy_wait(socket.as_raw_fd(), None).unwrap();
            let (rx_token, _) = socket
                                .receive()
                                .unwrap();

            let result = rx_token.consume(smoltcp::time::Instant::now(), |single_packet| {

                let frame = EthernetFrame::new_unchecked(&single_packet);
                if frame.ethertype() == EthernetProtocol::Ipv4 
                {
                    if let Ok(ipv4) = Ipv4Packet::new_checked(frame.payload()) {
                        Ok((frame.dst_addr(), frame.src_addr(), ipv4.total_len() as i32))
                    }
                    else {
                        Err(Error::Unrecognized)
                    }
                }
                else {
                    Err(Error::Unrecognized)
                }
            });

            // frame as tuple -> (destination mac, source mac, packet size)
            if let Ok(frame) = result {
                for device in &mut *all_devices
                {
                    let mac = device.addr_mac.parse::<EthernetAddress>().unwrap();
                    if mac == frame.0 || mac == frame.1
                    {
                        device.last_checked += frame.2;
                    }
                }
            }

            if check_throughput.elapsed().as_secs() > CHECK_THROUGHPUT 
            {
                for device in &mut *all_devices {
                    if device.is_blocked == 0 {
                    if device.last_checked > THROUGHPUT_THRESHOLD &&
                        device.last_last_checked > THROUGHPUT_THRESHOLD
                    {
                        device.is_watching = 1;
                        debug!("{} ---> watching", device.addr_ip);

                        let updated_row = diesel::update(
                            devices::table.filter(
                                devices::user_id.eq(device.user_id)))
                                .set(devices::is_watching.eq(1))
                                .execute(&conn);

                        let num_rows = diesel::update(
                            users::table.filter(
                                users::user_id.eq(device.user_id)))
                                .set(users::points.eq(users::points - 1))
                                .execute(&conn);

                        // CHECK IF USER HAS REACHED 0 POINTS!!
                        // IF SO, BLOCK ALL DEVICES
                        let points = users::table.filter(
                            users::user_id.eq(device.user_id))
                            .select(users::points)
                            .execute(&conn).unwrap();

                        if points == 0 {
                            debug!("Blocking device ip: {} -- {}", device.addr_ip, device.is_blocked);
                            network::block_ip(&device.addr_ip);

                            device.is_blocked == 1;
                            diesel::update(devices::table.filter(
                                    devices::user_id.eq(device.user_id)))
                                    .set(devices::is_blocked.eq(1))
                                    .execute(&conn);
                        }
                    }
                    else
                    {
                        let updated_row = diesel::update(
                            devices::table.filter(
                                devices::addr_mac.eq(device.addr_mac.clone())))
                                .set(devices::is_watching.eq(0))
                                .execute(&conn);

                        debug!("{} ---> Not watching ---> {}", device.addr_ip, device.last_checked);
                        device.is_watching = 0;
                    }
    
                    device.last_last_checked    = device.last_checked;
                    device.last_checked         = 0;
                    check_throughput            = Instant::now();
                }
                }

                break;
            }
        }
    }
}