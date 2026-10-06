package com.byocvpn.androidvpn

import android.app.Activity
import android.net.VpnService
import androidx.activity.result.ActivityResult
import app.tauri.annotation.ActivityCallback
import app.tauri.annotation.Command
import app.tauri.annotation.InvokeArg
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Invoke
import app.tauri.plugin.JSObject
import app.tauri.plugin.Plugin

@InvokeArg
class TunnelInterfaceRequest {
    lateinit var sessionName: String
    lateinit var addresses: Array<String>
    lateinit var dnsServers: Array<String>
    var mtu: Int = 1280
}

@InvokeArg
class ProtectSocketRequest {
    var socketFileDescriptor: Int = -1
}

@TauriPlugin
class AndroidVpnPlugin(private val activity: Activity) : Plugin(activity) {
    @Command
    fun requestPermission(invoke: Invoke) {
        val consentIntent = VpnService.prepare(activity)
        if (consentIntent == null) {
            invoke.resolve()
            return
        }
        startActivityForResult(invoke, consentIntent, "onPermissionResult")
    }

    @ActivityCallback
    fun onPermissionResult(invoke: Invoke, result: ActivityResult) {
        if (result.resultCode == Activity.RESULT_OK) {
            invoke.resolve()
        } else {
            invoke.reject("VPN permission was denied")
        }
    }

    @Command
    fun establish(invoke: Invoke) {
        val request = invoke.parseArgs(TunnelInterfaceRequest::class.java)
        ByocVpnService.runWithService(activity.applicationContext) { service ->
            try {
                val fileDescriptor = service.establishTunnelInterface(request)
                val response = JSObject()
                response.put("fileDescriptor", fileDescriptor)
                invoke.resolve(response)
            } catch (error: Exception) {
                invoke.reject(error.message ?: "Failed to establish the VPN interface")
            }
        }
    }

    @Command
    fun protect(invoke: Invoke) {
        val request = invoke.parseArgs(ProtectSocketRequest::class.java)
        if (ByocVpnService.protectSocket(request.socketFileDescriptor)) {
            invoke.resolve()
        } else {
            invoke.reject("Failed to exclude the tunnel socket from the VPN")
        }
    }

    @Command
    fun getStatus(invoke: Invoke) {
        val response = JSObject()
        response.put("isRunning", ByocVpnService.isRunning())
        invoke.resolve(response)
    }

    @Command
    fun stop(invoke: Invoke) {
        ByocVpnService.stopRunningService()
        invoke.resolve()
    }
}
