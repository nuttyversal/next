{ ... }:

{
	virtualisation = {
		oci-containers = {
			containers = {
				nutty-caddy = {
					image = "caddy:2.9";

					volumes = [
						# Stores issued TLS certificates.
						"/data/caddy/data:/data"

						# Stores Caddy configuration.
						"/data/caddy/config:/config"

						# Stores trusted CA certificates.
						"${../../certificates}:/trust"

						# Stores Caddyfile.
						"${../config}:/etc/caddy"
					];

					ports = [
						# HTTP
						"80:80"

						# HTTPS
						"443:443"

						# HTTP/3 (QUIC)
						"443:443/udp"
					];

					extraOptions = [
						# Caddy ships with HTTP/3 support enabled by default. To
						# improve the performance of this UDP based protocol, the
						# underlying quic-go library tries to increase the buffer
						# sizes for its socket. The NET_ADMIN capability allows it
						# to override the low default limits of the operating system
						# without having to change kernel parameters via sysctl.
						"--cap-add=NET_ADMIN"
					];
				};
			};
		};
	};

	networking = {
		firewall = {
			allowedTCPPorts = [
				# HTTP
				80

				# HTTPS
				443
			];

			allowedUDPPorts = [
				# HTTP/3
				443
			];
		};
	};
}
