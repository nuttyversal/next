{ modulesPath, ... }:

{
	imports = [
		(modulesPath + "/profiles/qemu-guest.nix")
	];

	boot = {
		loader = {
			grub = {
				device = "/dev/sda";
			};
		};

		initrd = {
			availableKernelModules = [
				"ata_piix"
				"uhci_hcd"
				"xen_blkfront"
				"vmw_pvscsi"
			];

			kernelModules = [
				"nvme"
			];
		};

		tmp = {
			cleanOnBoot = true;
		};
	};

	fileSystems = {
		"/" = {
			device = "/dev/sda1";
			fsType = "ext4";
		};
	};

	zramSwap = {
		enable = true;
	};
}
