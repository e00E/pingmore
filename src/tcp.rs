use std::{io::Result, net::SocketAddr, time::Instant};

use crate::deadline::deadline_to_timeout;

pub fn connect(address: SocketAddr, deadline: Option<Instant>) -> Result<()> {
    match deadline_to_timeout(deadline) {
        Some(timeout) => std::net::TcpStream::connect_timeout(&address, timeout)?,
        None => std::net::TcpStream::connect(address)?,
    };
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

    use crate::test::{LOCAL_TIMEOUT, REMOTE_IPV4, REMOTE_TIMEOUT};

    use super::*;

    fn local(bind_address: IpAddr) {
        let (sender, receiver) = std::sync::mpsc::sync_channel(0);
        let server = move || {
            let socket = std::net::TcpListener::bind((bind_address, 0)).unwrap();
            let port = socket.local_addr().unwrap().port();
            sender.send(port).unwrap();
            socket.accept().unwrap();
        };
        let thread = std::thread::spawn(server);
        let port = receiver.recv().unwrap();
        connect(
            (bind_address, port).into(),
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
        connect(
            (Ipv4Addr::LOCALHOST, 1).into(),
            Some(Instant::now() + LOCAL_TIMEOUT),
        )
        .unwrap_err();
    }

    // This test doesn't exist for ipv6 because the Cloudflare IPv6 address doesn't respond to HTTP.
    #[test]
    #[ignore]
    fn remote_ipv4() {
        connect(
            (REMOTE_IPV4, 80).into(),
            Some(Instant::now() + REMOTE_TIMEOUT),
        )
        .unwrap();
    }
}
