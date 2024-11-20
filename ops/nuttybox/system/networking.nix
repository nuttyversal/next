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
				# Trust Caddy's internal CA certificate in nuttygate.
				# /var/lib/caddy/.local/share/caddy/pki/authorities/local/root.crt
				''
					-----BEGIN CERTIFICATE-----
					MIIBpDCCAUqgAwIBAgIRAK21T113dh0jNjmTO42aSJwwCgYIKoZIzj0EAwIwMDEu
					MCwGA1UEAxMlQ2FkZHkgTG9jYWwgQXV0aG9yaXR5IC0gMjAyNCBFQ0MgUm9vdDAe
					Fw0yNDExMjAyMzMxMDJaFw0zNDA5MjkyMzMxMDJaMDAxLjAsBgNVBAMTJUNhZGR5
					IExvY2FsIEF1dGhvcml0eSAtIDIwMjQgRUNDIFJvb3QwWTATBgcqhkjOPQIBBggq
					hkjOPQMBBwNCAARjt8bU1QaG6giGvsTKAaMxkzOcanQs0tdrNILXiQeFmD7IHdky
					XGg0e3ZzdTUhgPtE0y8Y8p4HVBazd3vMOyBvo0UwQzAOBgNVHQ8BAf8EBAMCAQYw
					EgYDVR0TAQH/BAgwBgEB/wIBATAdBgNVHQ4EFgQUjQqbZXdQpptlLoenQ8gDc/rI
					VH4wCgYIKoZIzj0EAwIDSAAwRQIgKPI9pPkB/G4u8HJ933rM4UXs82o5sBmsspFT
					4bLcR7UCIQCz6/0saJNss2EVC1XsmBk6vzHQcTSfDZUBuGsKHWgLzw==
					-----END CERTIFICATE-----
				''
			];
		};
	};
}
