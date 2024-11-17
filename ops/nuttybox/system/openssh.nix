{ ... }:

{
	services = {
		openssh = {
			enable = true;

			hostKeys = [
				{
					path = "/sys/ssh/ed25519.key";
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

	networking = {
		firewall = {
			allowedTCPPorts = [ 22 ];
		};
	};
}
