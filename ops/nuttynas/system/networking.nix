{ ... }:

{
	networking = {
		hostName = "nuttynas";
		hostId = "8588b824";

		nameservers = [
			# Cloudflare
			"1.1.1.1"

			# Google
			"8.8.8.8"
		];

		useDHCP = true;

		firewall = {
			enable = true;
			allowedTCPPorts = [ 22 ];
		};
	};

	services = {
		openssh = {
			enable = true;

			hostKeys = [
				{
					path = "/host/ssh/ed25519.key";
					type = "ed25519";
				}
			];

			# Send 💓 every 60 seconds to prevent timeouts.
			extraConfig = "ClientAliveInterval 60";

			settings = {
				PermitRootLogin = "yes";
			};
		};
	};
}
