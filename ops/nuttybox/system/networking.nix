{ ... }:

{
	networking = {
		hostName = "nuttybox";
		hostId = "c5bb9702";

		hosts = {
			"10.100.0.1" = [ "nuttygate" ];
		};

		nameservers = [
			# Nuttyverse
			"10.100.0.1"

			# Cloudflare
			"1.1.1.1"

			# Google
			"8.8.8.8"
		];

		useDHCP = true;

		firewall = {
			enable = true;
		};

		wireguard = {
			interfaces = {
				wg0 = {
					ips = [ "10.100.0.2/24" ];
					privateKeyFile = "/host/wireguard/private";

					peers = [
						{
							publicKey = "/JuoCrhAu+x3x4yXiVxSd0Zd8iDOzxIQKMFNNIHLjkI=";
							endpoint = "nuttyver.se:51820";
							allowedIPs = [ "10.100.0.0/24" ];
							persistentKeepalive = 30;
						}
					];
				};
			};
		};
	};

	security = {
		pki = {
			certificateFiles = [
				../../certificates/caddy.crt
			];
		};
	};
}
