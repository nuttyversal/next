{ ... }:

{
	virtualisation = {
		oci-containers = {
			nutty-ca = {
				image = "smallstep/step-ca";
				volumes = [ "/data/certificates:/home/step" ];
				ports = [ "8443:9000" ];
			};
		};
	};

	networking = {
		firewall = {
			allowedTCPPorts = [ 8443 ];
		};
	};
}
