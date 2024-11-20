{
	modulesPath,
	...
}:

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

		kernel = {
			sysctl = {
				# Enable IP forwarding.
				"net.ipv4.ip_forward" = 1;
			};
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
