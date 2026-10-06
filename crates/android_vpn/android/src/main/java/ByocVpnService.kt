package com.byocvpn.androidvpn

import android.content.Context
import android.content.Intent
import android.net.VpnService
import android.os.Build
import android.os.Handler
import android.os.Looper

class ByocVpnService : VpnService() {
    companion object {
        private const val IPV4_DEFAULT_ROUTE = "0.0.0.0"
        private const val IPV6_DEFAULT_ROUTE = "::"

        private val mainHandler = Handler(Looper.getMainLooper())
        private val pendingActions = mutableListOf<(ByocVpnService) -> Unit>()

        @Volatile
        private var runningService: ByocVpnService? = null

        fun runWithService(context: Context, action: (ByocVpnService) -> Unit) {
            mainHandler.post {
                val service = runningService
                if (service != null) {
                    action(service)
                } else {
                    pendingActions.add(action)
                    context.startService(Intent(context, ByocVpnService::class.java))
                }
            }
        }

        fun protectSocket(socketFileDescriptor: Int): Boolean {
            val service = runningService ?: return false
            return service.protect(socketFileDescriptor)
        }

        fun isRunning(): Boolean = runningService != null

        fun stopRunningService() {
            mainHandler.post { runningService?.stopSelf() }
        }
    }

    override fun onCreate() {
        super.onCreate()
        runningService = this
    }

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        runPendingActions()
        return START_NOT_STICKY
    }

    override fun onRevoke() {
        runningService = null
        stopSelf()
    }

    override fun onDestroy() {
        runningService = null
        super.onDestroy()
    }

    fun establishTunnelInterface(request: TunnelInterfaceRequest): Int {
        val builder = Builder()
            .setSession(request.sessionName)
            .setMtu(request.mtu)

        var hasIpv4Address = false
        var hasIpv6Address = false
        for (address in request.addresses) {
            val (hostAddress, prefixLength) = splitCidr(address)
            builder.addAddress(hostAddress, prefixLength)
            if (hostAddress.contains(':')) {
                hasIpv6Address = true
            } else {
                hasIpv4Address = true
            }
        }
        if (hasIpv4Address) {
            builder.addRoute(IPV4_DEFAULT_ROUTE, 0)
        }
        if (hasIpv6Address) {
            builder.addRoute(IPV6_DEFAULT_ROUTE, 0)
        }

        for (dnsServer in request.dnsServers) {
            builder.addDnsServer(dnsServer)
        }

        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
            builder.setMetered(false)
        }

        val tunnelInterface = builder.establish()
            ?: throw IllegalStateException("VPN permission has not been granted")
        return tunnelInterface.detachFd()
    }

    private fun runPendingActions() {
        val actions = pendingActions.toList()
        pendingActions.clear()
        actions.forEach { action -> action(this) }
    }

    private fun splitCidr(address: String): Pair<String, Int> {
        val hostAddress = address.substringBefore('/')
        val defaultPrefixLength = if (hostAddress.contains(':')) 128 else 32
        val prefixLength = address.substringAfter('/', "").toIntOrNull() ?: defaultPrefixLength
        return Pair(hostAddress, prefixLength)
    }
}
