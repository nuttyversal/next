{ ... }:

{
	virtualisation = {
		podman = {
			enable = true;

			# Create an alias mapping docker to podman.
			dockerCompat = true;
		};

		oci-containers = {
			backend = "podman";

			containers = {
				watchtower = {
					image = "containrrr/watchtower";
					volumes = [ "/run/podman/podman.sock:/var/run/docker.sock" ];

					environment = {
						# Check for new updates every minute.
						WATCHTOWER_POLL_INTERVAL = "60";
					};
				};
			};
		};
	};
}
