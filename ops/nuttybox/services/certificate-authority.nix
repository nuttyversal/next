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
				password = "/host/tls/secrets/root-ca-password";

				address = "10.100.0.2:8443";
				dnsNames = [ "ca.nuttynet" ];

				authority = {
					provisioners = [
						{
							type = "JWK";
							name = "mail@nuttyver.se";
							key = {
								use = "sig";
								kty = "EC";
								kid = "provisioner-key-id";
								crv = "P-256";
								alg = "ES256";
								x = "70W6LRtewEivGA9sKKP2dMinnUFUia_rNrtLqWcX2i8";
								y = "C_6gMSLp6bReAuEGHxSdZ8zmRP1i5snFOLlODSBHyNw";
							};
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
