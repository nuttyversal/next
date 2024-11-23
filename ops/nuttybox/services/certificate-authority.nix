{ ... }:

{
	virtualisation = {
		oci-containers = {
			containers = {
				nutty-ca = {
					image = "smallstep/step-ca";
					volumes = [ "/data/certificates:/home/step" ];
					ports = [ "10.100.0.2:8443:9000" ];
				};
			};
		};
	};

	networking = {
		firewall = {
			allowedTCPPorts = [ 8443 ];
		};
	};
}
