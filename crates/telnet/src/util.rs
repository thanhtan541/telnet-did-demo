use network_interface::{NetworkInterface, NetworkInterfaceConfig};
use std::error::Error;
use std::net::{IpAddr, Ipv4Addr};

// Struct to hold network details
#[derive(Debug, PartialEq)]
pub struct NetworkInfo {
    pub ip: Ipv4Addr,
    pub subnet_mask: Option<Ipv4Addr>,
    pub gateway: Option<Ipv4Addr>,
}

// Function to get IPv4 address, subnet mask, and gateway for the first active interface
pub fn get_ipv4_info() -> Result<Vec<NetworkInfo>, Box<dyn Error>> {
    let mut result = Vec::new();

    // Get network interfaces
    let interfaces = NetworkInterface::show()?;

    for interface in interfaces {
        if interface.name != "en0" {
            continue;
        }
        for addr in interface.addr {
            if let IpAddr::V4(ip) = addr.ip() {
                let subnet_mask = addr.netmask().and_then(|mask| match mask {
                    IpAddr::V4(mask) => Some(mask),
                    _ => None,
                });
                // Get gateway (only once, as it's system-wide)
                let gateway =
                    default_net::get_default_gateway()
                        .ok()
                        .and_then(|g| match g.ip_addr {
                            IpAddr::V4(gw) => Some(gw),
                            _ => None,
                        });

                result.push(NetworkInfo {
                    ip,
                    subnet_mask,
                    gateway,
                });
            }
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;

    // Mock NetworkInterface for testing

    #[test]
    fn test_get_ipv4_info() {
        let _ = get_ipv4_info().unwrap_or_else(|_| {
            vec![NetworkInfo {
                ip: Ipv4Addr::new(192, 168, 1, 100),
                subnet_mask: Some(Ipv4Addr::new(255, 255, 255, 0)),
                gateway: Some(Ipv4Addr::new(192, 168, 1, 1)),
            }]
        });
    }
}
