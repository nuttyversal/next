{ ... }:

{
	networking = {
		hostName = "nuttygate";

		nameservers = [
			"2a01:4ff:ff00::add:1"
			"2a01:4ff:ff00::add:2"
			"185.12.64.1"
		];

		defaultGateway = "172.31.1.1";

		defaultGateway6 = {
			address = "fe80::1";
			interface = "eth0";
		};

		dhcpcd = {
			enable = true;
		};

		usePredictableInterfaceNames = false;

		interfaces = {
			eth0 = {
				ipv4 = {
					addresses = [
						{
							address = "5.78.127.247";
							prefixLength = 32;
						}
					];

					routes = [
						{
							address = "172.31.1.1";
							prefixLength = 32;
						}
					];
				};

				ipv6 = {
					addresses = [
						{
							address = "2a01:4ff:1f0:90fd::1";
							prefixLength = 64;
						}
						{
							address = "fe80::9400:3ff:fedc:32f8";
							prefixLength = 64;
						}
					];

					routes = [
						{
							address = "fe80::1";
							prefixLength = 128;
						}
					];
				};
			};
		};
	};

	services = {
		udev = {
			extraRules = ''
				ATTR{address}=="96:00:03:dc:32:f8", NAME="eth0"
			'';
		};
	};
}
