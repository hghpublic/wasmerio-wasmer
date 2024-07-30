use std::{
    net::{IpAddr, SocketAddr},
    sync::OnceLock,
    time::Duration,
};

use colored::Colorize;
use dialoguer::theme::ColorfulTheme;
use virtual_net::{
    DynVirtualNetworking, IpCidr, IpRoute, NetworkError, Result, StreamSecurity,
    UnsupportedVirtualNetworking, VirtualIcmpSocket, VirtualNetworking, VirtualRawSocket,
    VirtualTcpListener, VirtualTcpSocket, VirtualUdpSocket,
};

/// A custom implementation of the [`virtual_net::VirtualNetwork`] that asks users if they want to
/// use networking features at runtime.
#[derive(Debug, Clone)]
pub(crate) struct AskingNetworking {
    enable: OnceLock<Result<bool>>,
    capable: DynVirtualNetworking,
    unsupported: DynVirtualNetworking,
}

fn ask_user(fn_name: &str) -> Result<bool> {
    let theme = ColorfulTheme::default();

    println!("Networking needs to be enabled to call function '{fn_name}'.");
    if dialoguer::Confirm::with_theme(&theme)
        .with_prompt("Enable networking?")
        .interact()
        .map_err(|_| NetworkError::UnknownError)?
    {
        eprintln!(
            "{}: to enable networking by default, use the `--net` flag",
            "Info".bold()
        );
        Ok(true)
    } else {
        Ok(false)
    }
}

macro_rules! call {
    ($self: expr, $fn_name: ident, $( $arg: expr ),* ) => {

        let enable_networking = $self.enable.get_or_init(|| ask_user(stringify!($fn_name))).map_err(|_| NetworkError::UnknownError)?;

        if enable_networking {
            return $self.capable.$fn_name( $( $arg ),* ).await;
        } else {
            return $self.unsupported.$fn_name( $( $arg ),* ).await;
        }
    };

    ($self: expr, $fn_name: ident) => {

        let enable_networking = $self.enable.get_or_init(|| ask_user(stringify!($fn_name))).map_err(|_| NetworkError::UnknownError)?;

        if enable_networking {
            return $self.capable.$fn_name().await;
        } else {
            return $self.unsupported.$fn_name().await;
        }
    };
}

impl AskingNetworking {
    pub(crate) fn new(capable_networking: DynVirtualNetworking) -> Self {
        let enable_networking = OnceLock::new();

        Self {
            enable: enable_networking,
            capable: capable_networking,
            unsupported: std::sync::Arc::new(UnsupportedVirtualNetworking::default()),
        }
    }
}

/// An implementation of virtual networking
#[async_trait::async_trait]
#[allow(unused_variables)]
impl VirtualNetworking for AskingNetworking {
    /// Bridges this local network with a remote network, which is required in
    /// order to make lower level networking calls (such as UDP/TCP)
    async fn bridge(
        &self,
        network: &str,
        access_token: &str,
        security: StreamSecurity,
    ) -> Result<()> {
        call!(self, bridge, network, access_token, security);
    }

    /// Disconnects from the remote network essentially unbridging it
    async fn unbridge(&self) -> Result<()> {
        call!(self, unbridge);
    }

    /// Acquires an IP address on the network and configures the routing tables
    async fn dhcp_acquire(&self) -> Result<Vec<IpAddr>> {
        call!(self, dhcp_acquire);
    }

    /// Adds a static IP address to the interface with a netmask prefix
    async fn ip_add(&self, ip: IpAddr, prefix: u8) -> Result<()> {
        call!(self, ip_add, ip, prefix);
    }

    /// Removes a static (or dynamic) IP address from the interface
    async fn ip_remove(&self, ip: IpAddr) -> Result<()> {
        call!(self, ip_remove, ip);
    }

    /// Clears all the assigned IP addresses for this interface
    async fn ip_clear(&self) -> Result<()> {
        call!(self, ip_clear);
    }

    /// Lists all the IP addresses currently assigned to this interface
    async fn ip_list(&self) -> Result<Vec<IpCidr>> {
        call!(self, ip_list);
    }

    /// Returns the hardware MAC address for this interface
    async fn mac(&self) -> Result<[u8; 6]> {
        call!(self, mac);
    }

    /// Adds a default gateway to the routing table
    async fn gateway_set(&self, ip: IpAddr) -> Result<()> {
        call!(self, gateway_set, ip);
    }

    /// Adds a specific route to the routing table
    async fn route_add(
        &self,
        cidr: IpCidr,
        via_router: IpAddr,
        preferred_until: Option<Duration>,
        expires_at: Option<Duration>,
    ) -> Result<()> {
        call!(
            self,
            route_add,
            cidr,
            via_router,
            preferred_until,
            expires_at
        );
    }

    /// Removes a routing rule from the routing table
    async fn route_remove(&self, cidr: IpAddr) -> Result<()> {
        call!(self, route_remove, cidr);
    }

    /// Clears the routing table for this interface
    async fn route_clear(&self) -> Result<()> {
        call!(self, route_clear);
    }

    /// Lists all the routes defined in the routing table for this interface
    async fn route_list(&self) -> Result<Vec<IpRoute>> {
        call!(self, route_list);
    }

    /// Creates a low level socket that can read and write Ethernet packets
    /// directly to the interface
    async fn bind_raw(&self) -> Result<Box<dyn VirtualRawSocket + Sync>> {
        call!(self, bind_raw);
    }

    /// Lists for TCP connections on a specific IP and Port combination
    /// Multiple servers (processes or threads) can bind to the same port if they each set
    /// the reuse-port and-or reuse-addr flags
    async fn listen_tcp(
        &self,
        addr: SocketAddr,
        only_v6: bool,
        reuse_port: bool,
        reuse_addr: bool,
    ) -> Result<Box<dyn VirtualTcpListener + Sync>> {
        call!(self, listen_tcp, addr, only_v6, reuse_port, reuse_addr);
    }

    /// Opens a UDP socket that listens on a specific IP and Port combination
    /// Multiple servers (processes or threads) can bind to the same port if they each set
    /// the reuse-port and-or reuse-addr flags
    async fn bind_udp(
        &self,
        addr: SocketAddr,
        reuse_port: bool,
        reuse_addr: bool,
    ) -> Result<Box<dyn VirtualUdpSocket + Sync>> {
        call!(self, bind_udp, addr, reuse_port, reuse_addr);
    }

    /// Creates a socket that can be used to send and receive ICMP packets
    /// from a paritcular IP address
    async fn bind_icmp(&self, addr: IpAddr) -> Result<Box<dyn VirtualIcmpSocket + Sync>> {
        call!(self, bind_icmp, addr);
    }

    /// Opens a TCP connection to a particular destination IP address and port
    async fn connect_tcp(
        &self,
        addr: SocketAddr,
        peer: SocketAddr,
    ) -> Result<Box<dyn VirtualTcpSocket + Sync>> {
        call!(self, connect_tcp, addr, peer);
    }

    /// Performs DNS resolution for a specific hostname
    async fn resolve(
        &self,
        host: &str,
        port: Option<u16>,
        dns_server: Option<IpAddr>,
    ) -> Result<Vec<IpAddr>> {
        call!(self, resolve, host, port, dns_server);
    }
}
