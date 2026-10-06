#![cfg(target_os = "android")]

use std::os::fd::RawFd;

use serde::{Deserialize, Serialize, de::IgnoredAny};
use tauri::{
    Manager, Runtime,
    plugin::{Builder, PluginHandle, TauriPlugin, mobile::PluginInvokeError},
};

const PLUGIN_NAME: &str = "android-vpn";
const PLUGIN_IDENTIFIER: &str = "com.byocvpn.androidvpn";
const PLUGIN_CLASS_NAME: &str = "AndroidVpnPlugin";

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new(PLUGIN_NAME)
        .setup(|app_handle, plugin_api| {
            let plugin_handle =
                plugin_api.register_android_plugin(PLUGIN_IDENTIFIER, PLUGIN_CLASS_NAME)?;
            app_handle.manage(AndroidVpn(plugin_handle));
            Ok(())
        })
        .build()
}

pub trait AndroidVpnExt<R: Runtime> {
    fn android_vpn(&self) -> &AndroidVpn<R>;
}

impl<R: Runtime, T: Manager<R>> AndroidVpnExt<R> for T {
    fn android_vpn(&self) -> &AndroidVpn<R> {
        self.state::<AndroidVpn<R>>().inner()
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TunnelInterfaceRequest {
    pub session_name: String,
    pub addresses: Vec<String>,
    pub dns_servers: Vec<String>,
    pub mtu: u16,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct EstablishedTunnelInterface {
    file_descriptor: RawFd,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ProtectSocketRequest {
    socket_file_descriptor: RawFd,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct VpnServiceStatus {
    is_running: bool,
}

pub struct AndroidVpn<R: Runtime>(PluginHandle<R>);

impl<R: Runtime> Clone for AndroidVpn<R> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<R: Runtime> AndroidVpn<R> {
    pub async fn request_permission(&self) -> Result<(), PluginInvokeError> {
        self.0
            .run_mobile_plugin_async::<IgnoredAny>("requestPermission", ())
            .await?;
        Ok(())
    }

    pub async fn establish_tunnel_interface(
        &self,
        request: TunnelInterfaceRequest,
    ) -> Result<RawFd, PluginInvokeError> {
        let established_interface = self
            .0
            .run_mobile_plugin_async::<EstablishedTunnelInterface>("establish", request)
            .await?;
        Ok(established_interface.file_descriptor)
    }

    pub async fn protect_socket(
        &self,
        socket_file_descriptor: RawFd,
    ) -> Result<(), PluginInvokeError> {
        self.0
            .run_mobile_plugin_async::<IgnoredAny>(
                "protect",
                ProtectSocketRequest {
                    socket_file_descriptor,
                },
            )
            .await?;
        Ok(())
    }

    pub async fn is_running(&self) -> Result<bool, PluginInvokeError> {
        let service_status = self
            .0
            .run_mobile_plugin_async::<VpnServiceStatus>("getStatus", ())
            .await?;
        Ok(service_status.is_running)
    }

    pub async fn stop(&self) -> Result<(), PluginInvokeError> {
        self.0.run_mobile_plugin_async::<IgnoredAny>("stop", ()).await?;
        Ok(())
    }
}
