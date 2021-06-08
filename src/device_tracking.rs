use crate::{Pool};
use crate::schema::{devices};

use std::time::{Instant, Duration};
use std::os::unix::io::AsRawFd;

use smoltcp::phy::wait as phy_wait;
use smoltcp::phy::{Device, RxToken, RawSocket};
use smoltcp::wire::{EthernetAddress, PrettyPrinter, EthernetFrame, TcpPacket, Ipv4Packet, IpProtocol, EthernetProtocol};

use diesel::prelude::*;
use tokio::task;
use log::{error, debug};

pub struct DeviceToTrack {
    mac:                EthernetAddress,
    last_check:         u64,
    last_last_check:    u64,
    is_watching:        bool,
}

pub struct DeviceTracking {
    db: Pool,
    devices_not_init:   bool,
    all_devices:        Vec<DeviceToTrack>,
    check_streaming:    Instant,
    check_new_devices:  Instant,
}

impl DeviceTracking {
    pub fn new(db: Pool) -> Self {
        DeviceTracking {
            db,
            devices_not_init:   true,
            all_devices:        Vec::new(),
            check_streaming:    Instant::now(),
            check_new_devices:  Instant::now(),
        }
    }

    pub fn begin(&mut self) {
        const CHECK_STREAMING: u64          = 10;
        const CHECK_FOR_NEW_DEVICE: u64     = 60 * 60; // check for new devices every hr
        const STREAMING_THRESHOLD: u64      = 500_000;
        let mut socket                      = RawSocket::new("br0".as_ref()).unwrap();
             
        loop {
            let elapsed_new_device = self.check_new_devices.elapsed().as_secs();
            if self.devices_not_init || elapsed_new_device > CHECK_FOR_NEW_DEVICE {
                // let devices_to_track = 
                //     task::block_in_place(|| {
                    // query DB to get all devices that need to be tracked
                    let conn = 
                        self.db
                        .get()
                        .expect("couldn't get db connection from pool");
                        
                    let devices_from_db = 
                        devices::table
                        .filter(devices::is_tracked.eq(1))
                        .select(devices::addr_mac)
                        .load::<String>(&conn)
                            .map_err(|err| error!("Problem getting devices to track: {}", err))
                            .unwrap();

                //     devices_from_db
                // });

                // iter through all devices from DB that are to be tracked
                self.all_devices.clear();
                for dev in devices_from_db {
                    self.all_devices.push( DeviceToTrack {
                        mac: dev.parse::<EthernetAddress>().unwrap(),
                        last_check: 0,
                        last_last_check: 0,
                        is_watching: false,
                    });
                }

                // reset time to check for new devices
                self.check_new_devices  = Instant::now();
                self.devices_not_init   = false;
            }

            // ONLY TRACK DEVICES IF THERE ARE DEVICES TO TRACK
            if self.all_devices.len() > 0 {
                phy_wait(socket.as_raw_fd(), None).unwrap();
                let (rx_token, _) = socket.receive().unwrap();

                rx_token.consume(smoltcp::time::Instant::now(), |buffer| {

                    let frame = EthernetFrame::new_unchecked(&buffer);
                    if frame.ethertype() == EthernetProtocol::Ipv4 
                    {
                        for device in &mut self.all_devices {
                            if frame.dst_addr() == device.mac || frame.src_addr() == device.mac 
                            {
                                if let Ok(ipv4) = Ipv4Packet::new_checked(frame.payload()) {
                                    device.last_check += ipv4.total_len() as u64;
                                }
                            }
                        }
                    }
    
                    let elapsed_streaming = self.check_streaming.elapsed().as_secs();
                    if elapsed_streaming > CHECK_STREAMING 
                    {
                        for device in &mut self.all_devices {
                            if device.last_check > STREAMING_THRESHOLD &&
                                device.last_last_check > STREAMING_THRESHOLD
                            {
                                device.is_watching = true;
                                debug!("{} --->  {}", device.mac, device.is_watching);
                            }
                            else
                            {
                                device.is_watching = false;
                            }
    
                            device.last_last_check      = device.last_check;
                            device.last_check           = 0;
                            self.check_streaming        = Instant::now();
                        }
                    }
    
                    Ok(())
                }).unwrap();
            }
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
