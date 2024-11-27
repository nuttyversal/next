{ ... }:

{
	virtualisation = {
		oci-containers = {
			containers = {
				nutty-ca = {
					image = "smallstep/step-ca";
					volumes = [ "/data/certificates:/home/step" ];
					ports = [ "8443:443" ];
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
