{ ... }:

let
	nuttyKey = "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIIifhdP6vTWEGGvLgWK/93bTF8TGrlffo2o/V0krRbbP mail@nuttyver.se";
	nuttyPassword = "$6$8ZgyDGf6dDZGZKPs$XZes/zy.C59t47M.xnm2Ajgf3k1oenDp1NRicw5Qat8/UpmJPdO9doMwjyT3N.ayUSQ95VMNcfbGd7aTX/MDr0";
	emilyPassword = "$6$hw3KzK37WTVm6rrw$NyTOL9DkkSjNt/qx5n5HF5zli5iNwh8g9nnjHgadXnkMuLah/q0GO.TXZplwKrDZZWFgL7AV6BoPW.XWy8mFR0";
	rootPassword = "$6$Be73trjnXBl/aQqG$bJr64Tvq8tAsONCPg9Qzc2knUBphcdf315EOpbM73chiwR4bzt4hlfOOnFxWDszqNXQzg0s27IGWBsDsiRQ1d1";
in
{
	users = {
		# Immutability!
		mutableUsers = false;

		users = {
			nutty = {
				isNormalUser = true;
				description = "nuttyversal";
				home = "/home/nutty";
				homeMode = "700";
				extraGroups = [ "wheel" ];
				hashedPassword = nuttyPassword;

				openssh = {
					authorizedKeys = {
						keys = [ nuttyKey ];
					};
				};
			};

			emily = {
				isNormalUser = true;
				description = "wifey";
				home = "/home/emily";
				homeMode = "700";
				extraGroups = [ "wheel" ];
				hashedPassword = emilyPassword;
			};

			root = {
				hashedPassword = rootPassword;

				openssh = {
					authorizedKeys = {
						keys = [ nuttyKey ];
					};
				};
			};
		};

		groups = {
			family = {
				name = "family";
				members = [
					"nutty"
					"emily"
				];
			};
		};
	};
}
