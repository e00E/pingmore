use std::{
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
    time::Duration,
};

pub const LOCAL_TIMEOUT: Duration = Duration::from_secs(1);
pub const REMOTE_TIMEOUT: Duration = Duration::from_secs(10);

// We use Cloudflare's IP because they allow us to test tcp, udp, ping on the same address.
pub const REMOTE_IPV4: IpAddr = IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1));
pub const REMOTE_IPV6: IpAddr =
    IpAddr::V6(Ipv6Addr::new(0x2606, 0x4700, 0x4700, 0, 0, 0, 0, 0x1111));
