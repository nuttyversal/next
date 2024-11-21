{
	pkgs,
	...
}:

{
	services = {
		step-ca = {
			enable = true;
			address = "10.100.0.2";
			port = 8443;

			intermediatePasswordFile = "/host/tls/secrets/intermediate-ca-password";

			settings = {
				root = "/host/tls/certificates/root-ca.crt";
				crt = "/host/tls/certificates/intermediate-ca.crt";
				key = "/host/tls/secrets/intermediate-ca-key";

				address = "10.100.0.2:8443";
				dnsNames = [ "ca.nuttynet" ];

				authority = {
					provisioners = [
						{
							use = "sig";
							kty = "EC";
							kid = "Lhiq1Ry9i3PLdlttXmfH5iX7gr9NXVvu_ksI2InoC3Q";
							crv = "P-256";
							alg = "ES256";
							x = "Vdkcpm_IGH_TBPWST2E11XFFMBHwpHVaDLv2F1epJGY";
							y = "0ZYX8bb_uhSoWcOvOUxJv-Ao7mHFq90UItvJL6ElDNA";
						}
					];
				};
			};
		};
	};

	networking = {
		firewall = {
			allowedTCPPorts = [ 8443 ];
		};
	};

	environment = {
		systemPackages = [
			pkgs.step-cli
		];
	};
}
