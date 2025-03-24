let
	nutty = "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIIifhdP6vTWEGGvLgWK/93bTF8TGrlffo2o/V0krRbbP mail@nuttyver.se";
	nuttybox = "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIHD+1EU23M79Otwq1ZkB8DNfb4lEeHbsZh4nYdyGx2pr root@nuttybox";
in {
	"valheim-environment.age" = {
		publicKeys = [ nutty nuttybox ];
	};

	"vaultwarden-environment.age" = {
		publicKeys = [ nutty nuttybox ];
	};
}
