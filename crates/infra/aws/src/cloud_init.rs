pub(super) fn generate_wireguard_cloud_init(
    server_private_key: &str,
    client_public_key: &str,
) -> String {
    let wg_config = format!(
        r#"[Interface]
PrivateKey={server_private_key}
Address=10.66.66.1/24,fd86:ea04:1111::1/64
ListenPort=51820
PostUp = iptables -A FORWARD -i enX0 -j ACCEPT
PostUp = iptables -t nat -A POSTROUTING -o enX0 -j MASQUERADE
PostUp = ip6tables -A FORWARD -i enX0 -j ACCEPT
PostUp = ip6tables -t nat -A POSTROUTING -o enX0 -j MASQUERADE
PostDown = iptables -D FORWARD -i enX0 -j ACCEPT
PostDown = iptables -t nat -D POSTROUTING -o enX0 -j MASQUERADE
PostDown = ip6tables -D FORWARD -i enX0 -j ACCEPT
PostDown = ip6tables -t nat -D POSTROUTING -o enX0 -j MASQUERADE

[Peer]
PublicKey={client_public_key}
AllowedIPs=10.66.66.2/32,fd86:ea04:1111::2/128
"#
    );

    let user_data = format!(
        r#"#!/bin/bash
dnf install -y wireguard-tools iptables-services

cat>/etc/wireguard/wg0.conf<<'EOF'
{wg_config}
EOF

echo "net.ipv4.ip_forward=1">>/etc/sysctl.conf
echo "net.ipv6.conf.all.forwarding=1">>/etc/sysctl.conf
sysctl -p

systemctl start wg-quick@wg0
systemctl enable wg-quick@wg0
"#
    );
    return user_data;
}
