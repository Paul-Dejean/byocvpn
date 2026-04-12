use byocvpn_core::error::{ConfigurationError, Result};
use ipnet::IpNet;
use net_route::{Handle, Route};

use crate::routing::interface::get_ifindex;
use log::*;

pub async fn add_vpn_routes(iface_name: &str, server_ip: &str) -> Result<()> {
    info!(
        "Adding VPN routes for server {} via interface {}",
        server_ip, iface_name
    );

    let server_route = format!("{}/32", server_ip);
    let routes = [
        (server_route.as_str(), "default"),
        ("0.0.0.0/1", iface_name),
        ("128.0.0.0/1", iface_name),
        ("::/1", iface_name),
        ("8000::/1", iface_name),
    ];

    for (destination, interface) in routes.iter() {
        if let Err(error) = add_route(destination, interface).await {
            warn!("Failed to add route {} via {}: {}", destination, interface, error);
        }
    }

    info!("Finished adding VPN routes");
    Ok(())
}

pub async fn remove_vpn_routes(iface_name: &str, server_ip: &str) {
    info!(
        "Removing VPN routes for server {} via interface {}",
        server_ip, iface_name
    );

    let server_route = format!("{}/32", server_ip);
    let routes = [
        (server_route.as_str(), "default"),
        ("0.0.0.0/1", iface_name),
        ("128.0.0.0/1", iface_name),
        ("::/1", iface_name),
        ("8000::/1", iface_name),
    ];

    for (destination, interface) in routes.iter() {
        if let Err(error) = delete_route(destination, interface).await {
            warn!("Failed to remove route {} via {}: {}", destination, interface, error);
        }
    }

    info!("Finished removing VPN routes");
}

async fn add_route(destination: &str, interface: &str) -> Result<()> {
    debug!("Adding route: {} via {}", destination, interface);

    let subnet: IpNet = destination
        .parse()
        .map_err(|error| ConfigurationError::ParseError {
            value: "destination".to_string(),
            reason: format!("Invalid subnet {}: {}", destination, error),
        })?;

    let handle = Handle::new().map_err(|error| ConfigurationError::RouteConfiguration {
        reason: format!("failed to create route handle: {}", error),
    })?;
    let ifindex = get_ifindex(interface).await?;

    debug!("Interface index: {}", ifindex);

    let route = if interface == "default" {
        let default_route = handle
            .default_route()
            .await
            .map_err(|error| ConfigurationError::RouteConfiguration {
                reason: format!("failed to query default route: {}", error),
            })?
            .ok_or_else(|| ConfigurationError::RouteConfiguration {
                reason: "No default route found".to_string(),
            })?;
        let gateway =
            default_route
                .gateway
                .ok_or_else(|| ConfigurationError::RouteConfiguration {
                    reason: "Default route has no gateway".to_string(),
                })?;
        Route::new(subnet.addr(), subnet.prefix_len())
            .with_gateway(gateway)
            .with_ifindex(ifindex)
    } else {
        Route::new(subnet.addr(), subnet.prefix_len()).with_ifindex(ifindex)
    };

    debug!("Route configuration: {:?}", route);

    match handle.add(&route).await {
        Ok(_) => {
            debug!("Added route: {} via {}", destination, interface);
            Ok(())
        }
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            debug!("Route already exists: {} via {} (skipping)", destination, interface);
            Ok(())
        }
        Err(error) => {
            let error_message = format!(
                "Failed to add route {} via {}: {}",
                destination, interface, error
            );
            error!("{}", error_message);
            Err(ConfigurationError::RouteConfiguration {
                reason: error_message,
            }
            .into())
        }
    }
}

async fn delete_route(destination: &str, interface: &str) -> Result<()> {
    let subnet: IpNet =
        destination
            .parse()
            .map_err(|error| ConfigurationError::RouteConfiguration {
                reason: format!("Invalid subnet {}: {}", destination, error),
            })?;

    let ifindex = get_ifindex(interface).await?;
    let handle = Handle::new().map_err(|error| ConfigurationError::RouteConfiguration {
        reason: format!("failed to create route handle: {}", error),
    })?;

    let route = Route::new(subnet.addr(), subnet.prefix_len()).with_ifindex(ifindex);

    match handle.delete(&route).await {
        Ok(_) => {
            debug!("Deleted route: {} dev {}", destination, interface);
            Ok(())
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            debug!("Route not found: {} dev {} (already removed)", destination, interface);
            Ok(())
        }
        Err(error) => {
            let error_message = format!(
                "Failed to delete route {} dev {}: {}",
                destination, interface, error
            );
            error!("{}", error_message);
            Err(ConfigurationError::RouteConfiguration {
                reason: format!("Invalid subnet {}: {}", destination, error),
            }
            .into())
        }
    }
}

pub async fn update_server_host_route(server_ip: &str, last_gateway: &mut Option<std::net::IpAddr>) {
    let handle = match Handle::new() {
        Ok(handle) => handle,
        Err(error) => {
            error!("[RouteMonitor] Failed to create route handle: {}", error);
            return;
        }
    };

    let current_gateway = handle
        .default_route()
        .await
        .ok()
        .flatten()
        .and_then(|r| r.gateway);

    if current_gateway == *last_gateway {
        return;
    }

    info!(
        "[RouteMonitor] Gateway changed: {:?} -> {:?}",
        last_gateway, current_gateway
    );
    *last_gateway = current_gateway;

    let server_route = format!("{}/32", server_ip);

    if let Err(error) = delete_route(&server_route, "default").await {
        error!("[RouteMonitor] Failed to delete old host route: {}", error);
    }

    if let Err(error) = add_route(&server_route, "default").await {
        error!("[RouteMonitor] Failed to re-add host route: {}", error);
    } else {
        info!("[RouteMonitor] Host route for {} refreshed.", server_ip);
    }
}
