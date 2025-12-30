use byocvpn_core::error::{Error, Result};
use ipnet::IpNet;
use net_route::{Handle, Route};

use crate::routing::interface::get_ifindex;

pub async fn add_vpn_routes(iface_name: &str, server_ip: &str) -> Result<()> {
    println!(
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
        if let Err(e) = add_route(destination, interface).await {
            eprintln!(
                "Warning: Failed to add route {} via {}: {}",
                destination, interface, e
            );
            // Continue with other routes even if one fails
        }
    }

    println!("Finished adding VPN routes");
    Ok(())
}

pub async fn remove_vpn_routes(iface_name: &str, server_ip: &str) {
    println!(
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
        if let Err(e) = delete_route(destination, interface).await {
            eprintln!(
                "Warning: Failed to remove route {} via {}: {}",
                destination, interface, e
            );
        }
    }

    println!("Finished removing VPN routes");
}

async fn add_route(destination: &str, interface: &str) -> Result<()> {
    println!("Adding route: {} via {}", destination, interface);

    let subnet: IpNet = destination
        .parse()
        .map_err(|e| Error::RouteError(format!("Invalid subnet {}: {}", destination, e)))?;

    let handle = Handle::new()?;
    let ifindex = get_ifindex(interface).await?;

    println!("Interface index: {}", ifindex);

    // Build the route
    let route = if interface == "default" {
        // Set the default route
        let default_route = handle
            .default_route()
            .await?
            .ok_or_else(|| Error::RouteError("No default route found".to_string()))?;
        let gateway = default_route
            .gateway
            .ok_or_else(|| Error::RouteError("Default route has no gateway".to_string()))?;
        Route::new(subnet.addr(), subnet.prefix_len()).with_gateway(gateway)
    } else {
        Route::new(subnet.addr(), subnet.prefix_len()).with_ifindex(ifindex)
    };

    println!("Route configuration: {:?}", route);

    match handle.add(&route).await {
        Ok(_) => {
            println!("Added route: {} via {}", destination, interface);
            Ok(())
        }
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
            println!(
                "Route already exists: {} via {} (skipping)",
                destination, interface
            );
            Ok(())
        }
        Err(e) => {
            let err_msg = format!(
                "Failed to add route {} via {}: {}",
                destination, interface, e
            );
            eprintln!("{}", err_msg);
            Err(Error::RouteError(err_msg))
        }
    }
}

async fn delete_route(destination: &str, interface: &str) -> Result<()> {
    let subnet: IpNet = destination
        .parse()
        .map_err(|e| Error::RouteError(format!("Invalid subnet {}: {}", destination, e)))?;

    let ifindex = get_ifindex(interface).await?;
    let handle = Handle::new()?;

    // Build the route
    let route = Route::new(subnet.addr(), subnet.prefix_len()).with_ifindex(ifindex);

    // Delete the route
    match handle.delete(&route).await {
        Ok(_) => {
            println!("Deleted route: {} dev {}", destination, interface);
            Ok(())
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            println!(
                "Route not found: {} dev {} (already removed)",
                destination, interface
            );
            Ok(())
        }
        Err(e) => {
            let err_msg = format!(
                "Failed to delete route {} dev {}: {}",
                destination, interface, e
            );
            eprintln!("{}", err_msg);
            Err(Error::RouteError(err_msg))
        }
    }
}
