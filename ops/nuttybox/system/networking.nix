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
			certificates = [
				''
					Nuttyverse
					==========
					-----BEGIN CERTIFICATE-----
					MIIBszCCAVmgAwIBAgIQCscYRo0tw4ZV5A7R9pZ0STAKBggqhkjOPQQDAjA4MRYw
					FAYDVQQKEw1OdXR0eXZlcnNlIENBMR4wHAYDVQQDExVOdXR0eXZlcnNlIENBIFJv
					b3QgQ0EwHhcNMjQxMTIzMTY0NTU5WhcNMzQxMTIxMTY0NTU5WjA4MRYwFAYDVQQK
					Ew1OdXR0eXZlcnNlIENBMR4wHAYDVQQDExVOdXR0eXZlcnNlIENBIFJvb3QgQ0Ew
					WTATBgcqhkjOPQIBBggqhkjOPQMBBwNCAATugRACvdc+f5mel3UewjaL1fDMxNcK
					UjBLYl9kzU7poJlaVFB6wpqUZCwbO7pJvxBRg+14uuLo5HK5Tv7NwwBEo0UwQzAO
					BgNVHQ8BAf8EBAMCAQYwEgYDVR0TAQH/BAgwBgEB/wIBATAdBgNVHQ4EFgQUWCCP
					eExhc8IrY9aRVi4wFt4gy+cwCgYIKoZIzj0EAwIDSAAwRQIhALNWEyBfhK0K6hRE
					yNFuYKCctLNRv02ZblJiZsoc/klrAiAJOsJiyJYfL4q1nvilW1kDGJQHcbW6wXx7
					V2tF49AgzQ==
					-----END CERTIFICATE-----
				''
			];
		};
	};
}
