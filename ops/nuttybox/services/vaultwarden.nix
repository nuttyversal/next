{ ... }:

{
	services = {
		vaultwarden = {
			enable = true;

			# =# CREATE DATABASE vaultwarden;
			# =# CREATE USER vaultwarden WITH ENCRYPTED PASSWORD '<database-password>';
			# =# GRANT ALL ON DATABASE vaultwarden TO vaultwarden;
			# =# GRANT ALL PRIVILEGES ON DATABASE vaultwarden TO vaultwarden;
			# =# GRANT ALL ON SCHEMA public TO vaultwarden;
			dbBackend = "postgresql";

			# The database connection URL is stored in:
			environmentFile = "/run/secrets/vaultwarden-environment";

			config = {
				DOMAIN = "http://192.168.50.234";
				ROCKET_ADDRESS = "0.0.0.0";
				ROCKET_PORT = 9273;
				ROCKET_LOG = "critical";
				SIGNUPS_ALLOWED = true;
			};
		};
	};

	networking = {
		firewall = {
			allowedTCPPorts = [ 9273 ];
		};
	};

	age = {
		secrets = {
			vaultwarden-environment = {
				file = ../secrets/vaultwarden-environment.age;
				path = "/run/secrets/vaultwarden-environment";
				owner = "vaultwarden";
				group = "vaultwarden";
				mode = "600";
			};
		};
	};
}
