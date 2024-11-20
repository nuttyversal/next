{ ... }:

{
	networking = {
		hostName = "nuttygate";

		nameservers = [
			# Nuttyverse
			"127.0.0.1"

			# Hetzner (IPv6)
			"2a01:4ff:ff00::add:1"
			"2a01:4ff:ff00::add:2"

			# Hetzner (IPv4)
			"185.12.64.1"
			"185.12.64.2"

			# Cloudflare
			"1.1.1.1"

			# Google
			"8.8.8.8"
		];

		defaultGateway = "172.31.1.1";

		defaultGateway6 = {
			address = "fe80::1";
			interface = "eth0";
		};

		dhcpcd = {
			enable = true;
		};

		usePredictableInterfaceNames = false;

		interfaces = {
			eth0 = {
				ipv4 = {
					addresses = [
						{
							address = "5.78.127.247";
							prefixLength = 32;
						}
					];

					routes = [
						{
							address = "172.31.1.1";
							prefixLength = 32;
						}
					];
				};

				ipv6 = {
					addresses = [
						{
							address = "2a01:4ff:1f0:90fd::1";
							prefixLength = 64;
						}
						{
							address = "fe80::9400:3ff:fedc:32f8";
							prefixLength = 64;
						}
					];

					routes = [
						{
							address = "fe80::1";
							prefixLength = 128;
						}
					];
				};
			};
		};

		wireguard = {
			interfaces = {
				wg0 = {
					ips = [ "10.100.0.1/24" ];
					listenPort = 51820;
					privateKeyFile = "/host/wireguard/private";

					peers = [
						{
							publicKey = "JD3snzuU2Poi/SRnc96D11bZVGUwB0fY3k+cyzsXf2I=";
							allowedIPs = [ "10.100.0.2/32" ];
							persistentKeepalive = 30;
						}
					];
				};
			};
		};

		firewall = {
			enable = true;

			allowedTCPPorts = [ ];
			allowedUDPPorts = [ 51820 ];

			extraCommands = ''
				# Allow nuttynet machines to access Nuttyverse DNS.
				iptables -A INPUT -p udp -s 10.100.0.0/24 --dport 53 -j ACCEPT
				iptables -A INPUT -p tcp -s 10.100.0.0/24 --dport 53 -j ACCEPT

				# Allow nuttynet machines to access the internet by masquerading as
				# as the gateway's public IP via network address translation (NAT).
				iptables -t nat -A POSTROUTING -s 10.100.0.0/24 -o eth0 -j MASQUERADE

				# Allow forwarding between interfaces.
				iptables -A FORWARD -i wg0 -j ACCEPT
				iptables -A FORWARD -o wg0 -j ACCEPT
			'';
		};
	};

	services = {
		caddy = {
			enable = true;

			virtualHosts = {
				"vault.nuttynet" = {
					extraConfig = ''
						reverse_proxy 10.100.0.2:9273
					'';
				};
			};
		};

		coredns = {
			enable = true;

			config = ''
				.:53 {
					hosts {
						# Nutty Network Machines
						10.100.0.1 nuttygate
						10.100.0.2 nuttybox
						10.100.0.3 nuttynas
						10.100.0.4 nuttybook
						10.100.0.5 nuttytower

						# Nutty Network Services
						10.100.0.1 vault.nuttynet

						fallthrough
					}

					# Forward DNS queries to Hetzner, Cloudflare, and Google.
					forward . 2a01:4ff:ff00::add:1 2a01:4ff:ff00::add:2 185.12.64.1 185.12.64.2 1.1.1.1 8.8.8.8

					cache
				}
			'';
		};

		udev = {
			extraRules = ''
				# Ensure network interface is named eth0.
				ATTR{address}=="96:00:03:dc:32:f8", NAME="eth0"
			'';
		};
	};
}
