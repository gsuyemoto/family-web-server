use std::time::{Instant, Duration};
use std::os::unix::io::AsRawFd;

use smoltcp::phy::wait as phy_wait;
use smoltcp::phy::{Device, RxToken, RawSocket};
use smoltcp::wire::{EthernetAddress, PrettyPrinter, EthernetFrame, TcpPacket, Ipv4Packet, IpProtocol, EthernetProtocol};

pub struct Device2Track {
    mac: EthernetAddress,
    time: Instant,
    last_check: u64,
    last_last_check: u64,
    time_used: Duration,
    is_streaming: bool,
}

impl Device2Track {
    pub fn new(mac: &str) -> Self {
        Device2Track {
            mac: mac.parse::<EthernetAddress>().unwrap(),
            time: Instant::now(),
            last_check: 0,
            last_last_check: 0,
            time_used: Duration::from_secs(0),
            is_streaming: false,
        }
    }

    pub fn begin(&mut self) {
        const TIME_TO_CHECK: u64        = 10;
        const STREAMING_THRESHOLD: u64  = 500_000;

        let mut socket                  = RawSocket::new("br0".as_ref()).unwrap();
    
        loop {
            phy_wait(socket.as_raw_fd(), None).unwrap();
            let (rx_token, _) = socket.receive().unwrap();
            rx_token.consume(smoltcp::time::Instant::now(), |buffer| {
                let frame = EthernetFrame::new_unchecked(&buffer);
    
                if frame.ethertype() == EthernetProtocol::Ipv4 &&
                (frame.dst_addr() == self.mac || frame.src_addr() == self.mac)
                {
                    if let Ok(ipv4) = Ipv4Packet::new_checked(frame.payload()) {
                        self.last_check += ipv4.total_len() as u64;
                    }
                }
    
                let elapsed_time = self.time.elapsed().as_secs();
                if elapsed_time > TIME_TO_CHECK {
                    if self.last_check > STREAMING_THRESHOLD &&
                        self.last_last_check > STREAMING_THRESHOLD
                    {
                        self.is_streaming = true;
                    }
                    else
                    {
                        self.is_streaming = false;
                    }
    
                    println!("{} --->  {}", self.mac, self.is_streaming);
                    self.time               = Instant::now();
                    self.last_last_check    = self.last_check;
                    self.last_check         = 0;
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
