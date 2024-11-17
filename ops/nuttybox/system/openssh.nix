{ ... }:

{
	services = {
		openssh = {
			enable = true;

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
