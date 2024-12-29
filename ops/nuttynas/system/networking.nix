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

		samba = {
			enable = true;

			settings = {
				global = {
					"security" = "user";
					"workgroup" = "WORKGROUP";
					"server string" = "NAS";
					"server role" = "standalone server";
					"map to guest" = "bad user";
				};

				nutty = {
					"path" = "/nas/unshared/nutty";
					"valid users" = "nutty";
					"force user" = "nutty";
					"force group" = "users";
					"read only" = "no";
					"browseable" = "yes";
					"create mask" = "0644";
					"directory mask" = "0755";
				};

				emily = {
					"path" = "/nas/unshared/emily";
					"valid users" = "emily";
					"force user" = "emily";
					"force group" = "users";
					"read only" = "no";
					"browseable" = "yes";
					"create mask" = "0644";
					"directory mask" = "0755";
				};

				shared = {
					"path" = "/nas/shared";
					"valid users" = "@family";
					"force group" = "family";
					"read only" = "no";
					"browseable" = "yes";
					"create mask" = "0664";
					"directory mask" = "0775";
				};
			};
		};
	};
}
