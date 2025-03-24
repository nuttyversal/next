{ ... }:

{
	virtualisation = {
		oci-containers = {
			containers = {
				valheim = {
					image = "lloesche/valheim-server:latest";
					environmentFiles = [ "/run/secrets/valheim-environment" ];

					volumes = [
						"/data/valheim/saves:/home/steam/.config/unity3d/IronGate/Valheim"
						"/data/valheim/server:/home/steam/valheim"
						"/data/valheim/backups:/home/steam/backups"
					];

					extraOptions = [
						# Ensure the UID and GID of the steam user is the same as
						# the UID and GID configured for the host directories.
						"--userns=keep-id"
					];

					ports = [
						"2456:2456/udp"
						"2457:2457/udp"
						"2458:2458/udp"
					];
				};
			};
		};
	};

	networking = {
		firewall = {
			allowedUDPPorts = [
				2456
				2457
				2458
			];
		};
	};

	age = {
		secrets = {
			valheim-environment = {
				file = ../secrets/valheim-environment.age;
				path = "/run/secrets/valheim-environment";
				owner = "root";
				group = "root";
				mode = "600";
			};
		};
	};
}
