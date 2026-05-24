use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, UdpSocket};
use std::time::Duration;

use socket2::{Domain, Protocol, Socket, Type};

/// A server discovered via LAN multicast.
#[derive(Clone, Debug)]
pub struct DiscoveredServer {
    pub address: String,
    pub motd: String,
    pub player_count: String,
    pub max_players: String,
    pub version: String,
}

/// Bind a UDP socket to the Minecraft LAN discovery port and join the
/// multicast group. Sets a 500 ms read timeout so the caller can poll.
pub fn create_lan_socket() -> std::io::Result<UdpSocket> {
    let socket2_sock = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))?;
    socket2_sock.set_reuse_address(true)?;
    let addr: SocketAddr = SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 4445).into();
    socket2_sock.bind(&addr.into())?;
    let socket: UdpSocket = socket2_sock.into();

    if socket
        .join_multicast_v4(&Ipv4Addr::new(224, 0, 2, 60), &Ipv4Addr::UNSPECIFIED)
        .is_err()
    {
        tracing::warn!("LAN discovery: multicast not available, falling back to broadcast");
        socket.set_broadcast(true)?;
    }

    socket.set_read_timeout(Some(Duration::from_millis(500)))?;
    Ok(socket)
}

/// Parse a Minecraft Java Edition LAN broadcast packet.
///
/// The format is `[MOTD]<motd>[/MOTD][AD]<port>[/AD]` sent as UTF-8 bytes
/// to the multicast group `224.0.2.60:4445`.
pub fn parse_lan_packet(data: &[u8], src: SocketAddr) -> Option<DiscoveredServer> {
    let s = std::str::from_utf8(data).ok()?;

    let motd = s
        .find("[MOTD]")
        .and_then(|start| {
            let content_start = start + 6;
            s[content_start..].find("[/MOTD]").map(|end| {
                s[content_start..content_start + end].to_string()
            })
        })?;

    let port = s
        .find("[AD]")
        .and_then(|start| {
            let content_start = start + 4;
            s[content_start..].find("[/AD]").and_then(|end| {
                s[content_start..content_start + end].parse::<u16>().ok()
            })
        })?;

    Some(DiscoveredServer {
        address: format!("{}:{}", src.ip(), port),
        motd,
        player_count: "?".to_string(),
        max_players: "?".to_string(),
        version: "?".to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::SocketAddrV4;

    #[test]
    fn test_parse_standard_packet() {
        let data = b"[MOTD]A multiplayer world[/MOTD][AD]25565[/AD]";
        let src = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(192, 168, 1, 42), 25565));
        let result = parse_lan_packet(data, src).unwrap();
        assert_eq!(result.address, "192.168.1.42:25565");
        assert_eq!(result.motd, "A multiplayer world");
    }

    #[test]
    fn test_parse_packet_with_spaces() {
        let data = b"[MOTD]Good luck, have fun![/MOTD][AD]25565[/AD]";
        let src = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(10, 0, 0, 5), 25565));
        let result = parse_lan_packet(data, src).unwrap();
        assert_eq!(result.address, "10.0.0.5:25565");
        assert_eq!(result.motd, "Good luck, have fun!");
    }

    #[test]
    fn test_parse_missing_motd_tag() {
        let data = b"no tags here";
        let src = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 25565));
        assert!(parse_lan_packet(data, src).is_none());
    }

    #[test]
    fn test_parse_missing_ad_tag() {
        let data = b"[MOTD]Hello[/MOTD]no ad here";
        let src = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 25565));
        assert!(parse_lan_packet(data, src).is_none());
    }

    #[test]
    fn test_parse_not_utf8() {
        let data = &[0xff, 0xfe, 0x00];
        let src = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 25565));
        assert!(parse_lan_packet(data, src).is_none());
    }
}
