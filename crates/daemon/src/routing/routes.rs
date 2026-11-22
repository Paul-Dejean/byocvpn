use std::{io::ErrorKind, net::IpAddr};

use byocvpn_core::error::{Result, TunnelError};
use ipnet::IpNet;
use net_route::{Handle, Route};
use tracing::{debug, error, info, warn};

pub async fn add_vpn_routes(interface_index: u32, server_ip: &str) -> Result<()> {
    debug!("adding VPN routes for server {server_ip} via interface index {interface_index}");

    let server_route = format!("{server_ip}/32");

    if let Err(error) = add_default_gateway_route(&server_route).await {
        warn!(%error, "failed to add gateway route {server_route}");
    }

    for destination in ["0.0.0.0/1", "128.0.0.0/1", "::/1", "8000::/1"] {
        if let Err(error) = add_interface_route(destination, interface_index).await {
            warn!(%error, "failed to add interface route {destination} via index {interface_index}"
            );
        }
    }

    debug!("finished adding VPN routes");
    Ok(())
}

pub async fn remove_vpn_routes(interface_index: u32, server_ip: &str) {
    debug!("removing VPN routes for server {server_ip} via interface index {interface_index}");

    let server_route = format!("{server_ip}/32");

    if let Err(error) = delete_default_gateway_route(&server_route).await {
        warn!(%error, "failed to remove gateway route {server_route}");
    }

    for destination in ["0.0.0.0/1", "128.0.0.0/1", "::/1", "8000::/1"] {
        if let Err(error) = delete_interface_route(destination, interface_index).await {
            warn!(%error, "failed to remove interface route {destination} via index {interface_index}"
            );
        }
    }

    debug!("finished removing VPN routes");
}

async fn add_default_gateway_route(destination: &str) -> Result<()> {
    debug!("adding gateway route: {destination}");

    let subnet = parse_destination(destination)?;

    let handle = Handle::new().map_err(TunnelError::route("handle creation"))?;

    let default_route = handle
        .default_route()
        .await
        .map_err(TunnelError::route("default route lookup"))?
        .ok_or(TunnelError::NoDefaultRoute)?;

    let gateway = default_route
        .gateway
        .ok_or(TunnelError::DefaultRouteWithoutGateway)?;

    let route = Route::new(subnet.addr(), subnet.prefix_len()).with_gateway(gateway);

    debug!("route configuration: {route:?}");

    match handle.add(&route).await {
        Ok(()) => {
            debug!("added gateway route: {destination} via {gateway:?}");
            Ok(())
        }
        Err(error) if error.kind() == ErrorKind::AlreadyExists => {
            debug!("gateway route already exists: {destination} (skipping)");
            Ok(())
        }
        Err(error) => Err(TunnelError::route("addition")(error).into()),
    }
}

async fn add_interface_route(destination: &str, interface_index: u32) -> Result<()> {
    debug!("adding interface route: {destination} via index {interface_index}");

    let subnet = parse_destination(destination)?;

    let handle = Handle::new().map_err(TunnelError::route("handle creation"))?;

    let route = Route::new(subnet.addr(), subnet.prefix_len()).with_ifindex(interface_index);

    debug!("route configuration: {route:?}");

    match handle.add(&route).await {
        Ok(()) => {
            debug!("added interface route: {destination} via index {interface_index}");
            Ok(())
        }
        Err(error) if error.kind() == ErrorKind::AlreadyExists => {
            debug!(
                "interface route already exists: {destination} via index {interface_index} (skipping)"
            );
            Ok(())
        }
        Err(error) => Err(TunnelError::route("addition")(error).into()),
    }
}

async fn delete_default_gateway_route(destination: &str) -> Result<()> {
    debug!("deleting gateway route: {destination}");

    let subnet = parse_destination(destination)?;

    let handle = Handle::new().map_err(TunnelError::route("handle creation"))?;

    let default_route = handle
        .default_route()
        .await
        .map_err(TunnelError::route("default route lookup"))?
        .ok_or(TunnelError::NoDefaultRoute)?;

    let gateway = default_route
        .gateway
        .ok_or(TunnelError::DefaultRouteWithoutGateway)?;

    let route = Route::new(subnet.addr(), subnet.prefix_len()).with_gateway(gateway);

    match handle.delete(&route).await {
        Ok(()) => {
            debug!("deleted gateway route: {destination}");
            Ok(())
        }
        Err(error) if error.kind() == ErrorKind::NotFound || error.raw_os_error() == Some(3) => {
            debug!("gateway route not found: {destination} (already removed)");
            Ok(())
        }
        Err(error) => Err(TunnelError::route("deletion")(error).into()),
    }
}

async fn delete_interface_route(destination: &str, interface_index: u32) -> Result<()> {
    debug!("deleting interface route: {destination} via index {interface_index}");

    let subnet = parse_destination(destination)?;

    let handle = Handle::new().map_err(TunnelError::route("handle creation"))?;

    let route = Route::new(subnet.addr(), subnet.prefix_len()).with_ifindex(interface_index);

    match handle.delete(&route).await {
        Ok(()) => {
            debug!("deleted interface route: {destination} via index {interface_index}");
            Ok(())
        }
        Err(error) if error.kind() == ErrorKind::NotFound || error.raw_os_error() == Some(3) => {
            debug!(
                "interface route not found: {destination} via index {interface_index} (already removed)"
            );
            Ok(())
        }
        Err(error) => Err(TunnelError::route("deletion")(error).into()),
    }
}

pub async fn update_server_host_route(server_ip: &str, last_gateway: &mut Option<IpAddr>) {
    let handle = match Handle::new() {
        Ok(handle) => handle,
        Err(error) => {
            error!(%error, "failed to create route handle");
            return;
        }
    };

    let current_gateway = handle
        .default_route()
        .await
        .ok()
        .flatten()
        .and_then(|route| route.gateway);

    if current_gateway == *last_gateway {
        return;
    }

    info!(from = ?last_gateway, to = ?current_gateway, "default gateway changed");
    *last_gateway = current_gateway;

    let server_route = format!("{server_ip}/32");

    if let Err(error) = delete_default_gateway_route(&server_route).await {
        error!(%error, "failed to delete old host route");
    }

    if let Err(error) = add_default_gateway_route(&server_route).await {
        error!(%error, "failed to re-add host route");
    } else {
        debug!(%server_ip, "host route refreshed");
    }
}

fn parse_destination(destination: &str) -> Result<IpNet> {
    destination.parse().map_err(|source| {
        TunnelError::InvalidRouteDestination {
            destination: destination.to_string(),
            source,
        }
        .into()
    })
}
