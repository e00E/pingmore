mod deadline;
mod icmp;
mod tcp;
#[cfg(test)]
mod test;
mod udp;

use std::{
    io::ErrorKind,
    net::IpAddr,
    process::ExitCode,
    str::FromStr,
    time::{Duration, Instant},
};

use clap::{Parser, ValueEnum};

/// a flexible ping utility
///
/// pingmore determines the roundtrip latency to a target by pinging it. It differs from the traditional ping command by supporting more kinds (--kind) of pings. This is useful because none of the methods require special privileges on the local machine, and it still works when the remote machine doesn't respond to some kinds of ping for example when ICMP is blocked.
///
/// Examples:
/// - pingmore 1.1.1.1
///   traditional ping
/// - pingmore 1.1.1.1 --kind dns
///   ping by talking to a DNS server
/// - pingmore 1.1.1.1 --kind tcp --port 80
///   ping by talking to an HTTP server
/// - pingmore 1.1.1.1 --kind udp --port 53 --payload 0000000000010000000000000000000000
///   ping by talking to a UDP server, equivalent to the DNS example
#[derive(Debug, Parser)]
#[command(verbatim_doc_comment)]
struct Args {
    /// target ip address
    target: IpAddr,

    /// timeout in seconds, default is no timeout
    #[arg(short, long)]
    timeout: Option<f32>,

    /// kind of ping
    ///
    /// - dns: Send a DNS request to the target. Succeed when any response is received.
    /// - icmp: Send an ICMP echo request to the target. Succeed when the echo response is received.
    /// Use IPPROTO_ICMP to run as an unprivileged user. This isn't supported on all systems. You might have to edit the sysctl "net.ipv4.ping_group_range".
    /// - tcp: Establish a TCP connection to the target. Succeed when the connection is established.
    /// - udp: Send the payload to the target. Succeed when any response is received.
    #[arg(short, long, default_value = "icmp", verbatim_doc_comment)]
    kind: Kind,

    /// port for ping types dns (default 53), tcp, udp
    #[arg(short, long)]
    port: Option<u16>,

    /// hex encoded payload for udp ping, no "0x" prefix, default "00"
    #[arg(long)]
    payload: Option<UdpPayloadArgument>,
}

#[derive(Debug, Clone, ValueEnum)]
enum Kind {
    Dns,
    Icmp,
    Tcp,
    Udp,
}

#[derive(Clone)]
struct UdpPayloadArgument(Vec<u8>);

impl FromStr for UdpPayloadArgument {
    type Err = hex::FromHexError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Err(hex::FromHexError::InvalidStringLength);
        }
        Ok(Self(hex::decode(s)?))
    }
}

impl std::fmt::Debug for UdpPayloadArgument {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let hex = hex::encode(self.0.as_slice());
        f.debug_tuple("UdpPayloadArgument").field(&hex).finish()
    }
}

fn main() -> ExitCode {
    let mut args = Args::parse();
    match args.kind {
        Kind::Dns if args.port.is_none() => {
            args.port = Some(53);
        }
        Kind::Icmp if args.port.is_some() => {
            println!("ICMP ping does not use port.");
            return ExitCode::FAILURE;
        }
        Kind::Icmp if args.payload.is_some() => {
            println!("ICMP ping does not use payload.");
            return ExitCode::FAILURE;
        }
        Kind::Tcp if args.port.is_none() => {
            println!("TCP ping needs port.");
            return ExitCode::FAILURE;
        }
        Kind::Tcp if args.payload.is_some() => {
            println!("TCP ping does not use payload.");
            return ExitCode::FAILURE;
        }
        Kind::Udp if args.port.is_none() => {
            println!("UDP ping needs port.");
            return ExitCode::FAILURE;
        }
        Kind::Udp if args.payload.is_none() => {
            args.payload = Some(UdpPayloadArgument(vec![0]));
        }
        _ => (),
    };

    let start = Instant::now();
    let deadline = args.timeout.map(|t| start + Duration::from_secs_f32(t));
    let result = match args.kind {
        Kind::Icmp => icmp::echo(args.target, deadline),
        Kind::Tcp => tcp::connect((args.target, args.port.unwrap()).into(), deadline),
        Kind::Udp => udp::echo(
            (args.target, args.port.unwrap()).into(),
            args.payload.as_ref().unwrap().0.as_slice(),
            deadline,
        ),
        Kind::Dns => udp::dns((args.target, args.port.unwrap()).into(), deadline),
    };
    let elapsed = start.elapsed();

    let seconds = elapsed.as_secs_f32();
    let millis = (seconds * 1000.).round();
    match result {
        Ok(()) => {
            println!("success in {millis} ms ({seconds:.3e} s)");
            ExitCode::SUCCESS
        }
        Err(err) if is_timeout(&err.kind()) => {
            println!("timeout in {millis} ms ({seconds:.3e} s");
            ExitCode::FAILURE
        }
        Err(err) => {
            println!("error in {millis} ms ({seconds:.3e} s: {err}",);
            ExitCode::FAILURE
        }
    }
}

fn is_timeout(err: &ErrorKind) -> bool {
    matches!(err, ErrorKind::WouldBlock | ErrorKind::TimedOut)
}
