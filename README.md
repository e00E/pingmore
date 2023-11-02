a flexible ping utility

pingmore determines the roundtrip latency to a target by pinging it. It differs from the traditional ping command by supporting more kinds (--kind) of pings. This is useful because none of the methods require special privileges on the local machine, and it still works when the remote machine doesn't respond to some kinds of ping for example when ICMP is blocked.

Examples:
- `pingmore 1.1.1.1` traditional ping
- `pingmore 1.1.1.1 --kind dns` ping by talking to a DNS server
- `pingmore 1.1.1.1 --kind tcp --port 80` ping by talking to an HTTP server
- `pingmore 1.1.1.1 --kind udp --port 53 --payload 0000000000010000000000000000000000` ping by talking to a UDP server, equivalent to the DNS example

```
Usage: pingmore [OPTIONS] <TARGET>

Arguments:
  <TARGET>
          target ip address

Options:
  -t, --timeout <TIMEOUT>
          timeout in seconds, default is no timeout

  -k, --kind <KIND>
          kind of ping

          - dns: Send a DNS request to the target. Succeed when any response is received.
          - icmp: Send an ICMP echo request to the target. Succeed when the echo response is received.
          Use IPPROTO_ICMP to run as an unprivileged user. This isn't supported on all systems. You might have to edit the sysctl "net.ipv4.ping_group_range".
          - tcp: Establish a TCP connection to the target. Succeed when the connection is established.
          - udp: Send the payload to the target. Succeed when any response is received.

          [default: icmp]
          [possible values: dns, icmp, tcp, udp]

  -p, --port <PORT>
          port for ping types dns (default 53), tcp, udp

      --payload <PAYLOAD>
          hex encoded payload for udp ping, no "0x" prefix, default "00"

  -h, --help
          Print help (see a summary with '-h')
```
