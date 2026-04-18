use byocvpn_core::error::{ConfigurationError, Result};
use ipnet::IpNet;
use log::*;
use net_route::{Handle, Route};

use crate::routing::interface::get_interface_index;

pub async fn add_vpn_routes(interface_name: &str, server_ip: &str) -> Result<()> {
    info!(
        "Adding VPN routes for server {} via interface {}",
        server_ip, interface_name
    );

    let server_route = format!("{}/32", server_ip);

    if let Err(error) = add_default_gateway_route(&server_route).await {
        warn!("Failed to add gateway route {}: {}", server_route, error);
    }

    for destination in ["0.0.0.0/1", "128.0.0.0/1", "::/1", "8000::/1"] {
        if let Err(error) = add_interface_route(destination, interface_name).await {
            warn!(
                "Failed to add interface route {} via {}: {}",
                destination, interface_name, error
            );
        }
    }

    info!("Finished adding VPN routes");
    Ok(())
}

pub async fn remove_vpn_routes(interface_name: &str, server_ip: &str) {
    info!(
        "Removing VPN routes for server {} via interface {}",
        server_ip, interface_name
    );

    let server_route = format!("{}/32", server_ip);

    if let Err(error) = delete_default_gateway_route(&server_route).await {
        warn!("Failed to remove gateway route {}: {}", server_route, error);
    }

    for destination in ["0.0.0.0/1", "128.0.0.0/1", "::/1", "8000::/1"] {
        if let Err(error) = delete_interface_route(destination, interface_name).await {
            warn!(
                "Failed to remove interface route {} via {}: {}",
                destination, interface_name, error
            );
        }
    }

    info!("Finished removing VPN routes");
}

async fn add_default_gateway_route(destination: &str) -> Result<()> {
    debug!("Adding gateway route: {}", destination);

    let subnet: IpNet = destination
        .parse()
        .map_err(|error| ConfigurationError::ParseError {
            value: "destination".to_string(),
            reason: format!("Invalid subnet {}: {}", destination, error),
        })?;

    let handle = Handle::new().map_err(|error| ConfigurationError::RouteConfiguration {
        reason: format!("failed to create route handle: {}", error),
    })?;

    let default_route = handle
        .default_route()
        .await
        .map_err(|error| ConfigurationError::RouteConfiguration {
            reason: format!("failed to query default route: {}", error),
        })?
        .ok_or_else(|| ConfigurationError::RouteConfiguration {
            reason: "No default route found".to_string(),
        })?;

    let gateway = default_route
        .gateway
        .ok_or_else(|| ConfigurationError::RouteConfiguration {
            reason: "Default route has no gateway".to_string(),
        })?;

    let route = Route::new(subnet.addr(), subnet.prefix_len()).with_gateway(gateway);

    debug!("Route configuration: {:?}", route);

    match handle.add(&route).await {
        Ok(_) => {
            debug!("Added gateway route: {} via {:?}", destination, gateway);
            Ok(())
        }
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            debug!("Gateway route already exists: {} (skipping)", destination);
            Ok(())
        }
        Err(error) => {
            let error_message = format!("Failed to add gateway route {}: {}", destination, error);
            error!("{}", error_message);
            Err(ConfigurationError::RouteConfiguration {
                reason: error_message,
            }
            .into())
        }
    }
}

async fn add_interface_route(destination: &str, interface_name: &str) -> Result<()> {
    debug!(
        "Adding interface route: {} via {}",
        destination, interface_name
    );

    let subnet: IpNet = destination
        .parse()
        .map_err(|error| ConfigurationError::ParseError {
            value: "destination".to_string(),
            reason: format!("Invalid subnet {}: {}", destination, error),
        })?;

    let handle = Handle::new().map_err(|error| ConfigurationError::RouteConfiguration {
        reason: format!("failed to create route handle: {}", error),
    })?;

    let ifindex = get_interface_index(interface_name)?;
    let route = Route::new(subnet.addr(), subnet.prefix_len()).with_ifindex(ifindex);

    debug!("Route configuration: {:?}", route);

    match handle.add(&route).await {
        Ok(_) => {
            debug!(
                "Added interface route: {} via {}",
                destination, interface_name
            );
            Ok(())
        }
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            debug!(
                "Interface route already exists: {} via {} (skipping)",
                destination, interface_name
            );
            Ok(())
        }
        Err(error) => {
            let error_message = format!(
                "Failed to add interface route {} via {}: {}",
                destination, interface_name, error
            );
            error!("{}", error_message);
            Err(ConfigurationError::RouteConfiguration {
                reason: error_message,
            }
            .into())
        }
    }
}

async fn delete_default_gateway_route(destination: &str) -> Result<()> {
    debug!("Deleting gateway route: {}", destination);

    let subnet: IpNet =
        destination
            .parse()
            .map_err(|error| ConfigurationError::RouteConfiguration {
                reason: format!("Invalid subnet {}: {}", destination, error),
            })?;

    let handle = Handle::new().map_err(|error| ConfigurationError::RouteConfiguration {
        reason: format!("failed to create route handle: {}", error),
    })?;

    let default_route = handle
        .default_route()
        .await
        .map_err(|error| ConfigurationError::RouteConfiguration {
            reason: format!("failed to query default route: {}", error),
        })?
        .ok_or_else(|| ConfigurationError::RouteConfiguration {
            reason: "No default route found".to_string(),
        })?;

    let gateway = default_route
        .gateway
        .ok_or_else(|| ConfigurationError::RouteConfiguration {
            reason: "Default route has no gateway".to_string(),
        })?;

    let route = Route::new(subnet.addr(), subnet.prefix_len()).with_gateway(gateway);

    match handle.delete(&route).await {
        Ok(_) => {
            debug!("Deleted gateway route: {}", destination);
            Ok(())
        }
        Err(error)
            if error.kind() == std::io::ErrorKind::NotFound || error.raw_os_error() == Some(3) =>
        {
            debug!("Gateway route not found: {} (already removed)", destination);
            Ok(())
        }
        Err(error) => {
            let error_message =
                format!("Failed to delete gateway route {}: {}", destination, error);
            error!("{}", error_message);
            Err(ConfigurationError::RouteConfiguration {
                reason: error_message,
            }
            .into())
        }
    }
}

async fn delete_interface_route(destination: &str, interface_name: &str) -> Result<()> {
    debug!(
        "Deleting interface route: {} via {}",
        destination, interface_name
    );

    let subnet: IpNet =
        destination
            .parse()
            .map_err(|error| ConfigurationError::RouteConfiguration {
                reason: format!("Invalid subnet {}: {}", destination, error),
            })?;

    let handle = Handle::new().map_err(|error| ConfigurationError::RouteConfiguration {
        reason: format!("failed to create route handle: {}", error),
    })?;

    let ifindex = get_interface_index(interface_name)?;
    let route = Route::new(subnet.addr(), subnet.prefix_len()).with_ifindex(ifindex);

    match handle.delete(&route).await {
        Ok(_) => {
            debug!(
                "Deleted interface route: {} via {}",
                destination, interface_name
            );
            Ok(())
        }
        Err(error)
            if error.kind() == std::io::ErrorKind::NotFound || error.raw_os_error() == Some(3) =>
        {
            debug!(
                "Interface route not found: {} via {} (already removed)",
                destination, interface_name
            );
            Ok(())
        }
        Err(error) => {
            let error_message = format!(
                "Failed to delete interface route {} via {}: {}",
                destination, interface_name, error
            );
            error!("{}", error_message);
            Err(ConfigurationError::RouteConfiguration {
                reason: error_message,
            }
            .into())
        }
    }
}

pub async fn update_server_host_route(
    server_ip: &str,
    last_gateway: &mut Option<std::net::IpAddr>,
) {
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

    if let Err(error) = delete_default_gateway_route(&server_route).await {
        error!("[RouteMonitor] Failed to delete old host route: {}", error);
    }

    if let Err(error) = add_default_gateway_route(&server_route).await {
        error!("[RouteMonitor] Failed to re-add host route: {}", error);
    } else {
        info!("[RouteMonitor] Host route for {} refreshed.", server_ip);
    }
}
