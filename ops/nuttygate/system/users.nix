{ ... }:

let
	nuttyKey = "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIIifhdP6vTWEGGvLgWK/93bTF8TGrlffo2o/V0krRbbP mail@nuttyver.se";
in
{
	users = {
		# Immutability!
		mutableUsers = false;

		users = {
			root = {
				openssh = {
					authorizedKeys = {
						keys = [ nuttyKey ];
					};
				};
			};
		};
	};
}
