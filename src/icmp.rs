use std::{
    io::{Error, ErrorKind, Read, Result},
    net::{IpAddr, SocketAddr},
    time::Instant,
};

use crate::deadline::deadline_to_timeout;

pub fn echo(address: IpAddr, deadline: Option<Instant>) -> Result<()> {
    // https://lwn.net/Articles/420800/
    // https://en.wikipedia.org/wiki/ICMPv6

    let (domain, protocol, icmp_request_type, icmp_expected_response_type) = match address {
        IpAddr::V4(_) => (socket2::Domain::IPV4, socket2::Protocol::ICMPV4, 8, 0),
        IpAddr::V6(_) => (socket2::Domain::IPV6, socket2::Protocol::ICMPV6, 128, 129),
    };
    let mut socket = socket2::Socket::new(domain, socket2::Type::DGRAM, Some(protocol))?;
    // Port does not matter.
    socket.connect(&SocketAddr::from((address, 0)).into())?;
    let request: [u8; 8] = [icmp_request_type, 0, 0, 0, 0, 0, 0, 0];
    socket.set_write_timeout(deadline_to_timeout(deadline))?;
    socket.send(&request)?;
    let mut response = [0u8; 8];
    socket.set_read_timeout(deadline_to_timeout(deadline))?;
    socket.read_exact(&mut response)?;
    let icmp_actual_response_type = response[0];
    if icmp_actual_response_type != icmp_expected_response_type {
        let message = format!(
            "unexpected ICMP response type {}",
            icmp_actual_response_type
        );
        return Err(Error::new(ErrorKind::Other, message));
    };
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::net::{Ipv4Addr, Ipv6Addr};

    use crate::test::{LOCAL_TIMEOUT, REMOTE_IPV4, REMOTE_IPV6, REMOTE_TIMEOUT};

    use super::*;

    #[test]
    fn local_ipv4() {
        echo(
            Ipv4Addr::LOCALHOST.into(),
            Some(Instant::now() + LOCAL_TIMEOUT),
        )
        .unwrap();
    }

    #[test]
    fn local_ipv6() {
        echo(
            Ipv6Addr::LOCALHOST.into(),
            Some(Instant::now() + LOCAL_TIMEOUT),
        )
        .unwrap();
    }

    #[test]
    #[ignore]
    fn remote_ipv4() {
        echo(REMOTE_IPV4, Some(Instant::now() + REMOTE_TIMEOUT)).unwrap();
    }

    #[test]
    #[ignore]
    fn remote_ipv6() {
        echo(REMOTE_IPV6, Some(Instant::now() + REMOTE_TIMEOUT)).unwrap();
    }
}
