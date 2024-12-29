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
			allowPing = true;

			allowedTCPPorts = [
				# SSH
				22

				# Syncthing GUI
				8384

				# Syncthing (syncing)
				22000
			];

			allowedUDPPorts = [
				# Syncthing (syncing)
				22000

				# Syncthing (discovery)
				21027
			];
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
			openFirewall = true;

			settings = {
				global = {
					"security" = "user";
					"workgroup" = "WORKGROUP";
					"server string" = "NAS";
					"server role" = "standalone server";
					"map to guest" = "bad user";
				};

				Nutty = {
					"path" = "/nas/unshared/nutty";
					"valid users" = "nutty";
					"force user" = "nutty";
					"force group" = "users";
					"read only" = "no";
					"browseable" = "yes";
					"create mask" = "0644";
					"directory mask" = "0755";
				};

				Emily = {
					"path" = "/nas/unshared/emily";
					"valid users" = "emily";
					"force user" = "emily";
					"force group" = "users";
					"read only" = "no";
					"browseable" = "yes";
					"create mask" = "0644";
					"directory mask" = "0755";
				};

				Shared = {
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

		# Used to advertise shares to Windows hosts.
		samba-wsdd = {
			enable = true;
			openFirewall = true;
		};

		# File synchronization between PCs and NAS.
		syncthing = {
			enable = true;
			openDefaultPorts = true;

			key = "/host/syncthing/key.pem";
			cert = "/host/syncthing/cert.pem";

			settings = {
				gui = {
					user = "admin";
					password = "admin";
				};

				devices = {
					"MLE-PC" = {
						id = "OXFBHGP-UVHO7WL-NWP3YIN-IEGI5FD-2DSFIDS-SL27CA4-67VSWUP-HS7NFQV";
					};
				};

				folders = {
					"Documents" = {
						path = "/nas/unshared/emily/Documents";
						devices = [ "MLE-PC" ];
					};

					"Music" = {
						path = "/nas/unshared/emily/Music";
						devices = [ "MLE-PC" ];
					};

					"MusicBee" = {
						path = "/nas/unshared/emily/MusicBee";
						devices = [ "MLE-PC" ];
					};

					"Screenshots" = {
						path = "/nas/unshared/emily/Screenshots";
						devices = [ "MLE-PC" ];
					};

					"Videos" = {
						path = "/nas/unshared/emily/Videos";
						devices = [ "MLE-PC" ];
					};

					"Game Videos" = {
						path = "/nas/unshared/emily/Game Videos";
						devices = [ "MLE-PC" ];
					};
				};
			};
		};
	};

	systemd = {
		services = {
			syncthing = {
				environment = {
					# Disable default ~/Sync folder.
					STNODEFAULTFOLDER = "true";
				};
			};
		};
	};
}
