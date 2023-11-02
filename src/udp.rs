use std::{
    io::Result,
    net::{Ipv4Addr, Ipv6Addr, SocketAddr},
    time::Instant,
};

use crate::deadline::deadline_to_timeout;

pub fn echo(address: SocketAddr, data: &[u8], deadline: Option<Instant>) -> Result<()> {
    assert!(!data.is_empty());

    let bind_address: SocketAddr = match address {
        SocketAddr::V4(_) => (Ipv4Addr::UNSPECIFIED, 0).into(),
        SocketAddr::V6(_) => (Ipv6Addr::UNSPECIFIED, 0).into(),
    };
    let socket = std::net::UdpSocket::bind(bind_address)?;
    socket.connect(address)?;
    socket.set_write_timeout(deadline_to_timeout(deadline))?;
    socket.send(data)?;
    socket.set_read_timeout(deadline_to_timeout(deadline))?;
    socket.recv(&mut [0u8; 1])?;
    Ok(())
}

pub fn dns(address: SocketAddr, deadline: Option<Instant>) -> Result<()> {
    // minimal valid dns request
    let request = [0u8, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    echo(address, &request, deadline)
}

#[cfg(test)]
mod tests {
    use std::net::IpAddr;

    use crate::test::{LOCAL_TIMEOUT, REMOTE_IPV4, REMOTE_IPV6, REMOTE_TIMEOUT};

    use super::*;

    fn local(bind_address: IpAddr) {
        let (sender, receiver) = std::sync::mpsc::sync_channel(0);
        let server = move || {
            let socket = std::net::UdpSocket::bind((bind_address, 0)).unwrap();
            let port = socket.local_addr().unwrap().port();
            sender.send(port).unwrap();
            let (_, address) = socket.recv_from(&mut [0]).unwrap();
            socket.send_to(&[0], address).unwrap();
        };
        let thread = std::thread::spawn(server);
        let port = receiver.recv().unwrap();
        echo(
            (bind_address, port).into(),
            &[0],
            Some(Instant::now() + LOCAL_TIMEOUT),
        )
        .unwrap();
        thread.join().unwrap();
    }

    #[test]
    fn local_ipv4() {
        local(Ipv4Addr::LOCALHOST.into());
    }

    #[test]
    fn local_ipv6() {
        local(Ipv6Addr::LOCALHOST.into());
    }

    #[test]
    fn local_not_listening() {
        echo(
            (Ipv4Addr::LOCALHOST, 1).into(),
            &[0],
            Some(Instant::now() + LOCAL_TIMEOUT),
        )
        .unwrap_err();
    }

    #[test]
    #[ignore]
    fn remote_ipv4() {
        dns(
            (REMOTE_IPV4, 53).into(),
            Some(Instant::now() + REMOTE_TIMEOUT),
        )
        .unwrap();
    }

    #[test]
    #[ignore]
    fn remote_ipv6() {
        dns(
            (REMOTE_IPV6, 53).into(),
            Some(Instant::now() + REMOTE_TIMEOUT),
        )
        .unwrap();
    }
}
