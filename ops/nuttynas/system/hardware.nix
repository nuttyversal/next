{
	config,
	lib,
	modulesPath,
	...
}:

{
	imports = [
		(modulesPath + "/installer/scan/not-detected.nix")
	];

	boot = {
		loader = {
			systemd-boot = {
				enable = true;
			};

			efi = {
				canTouchEfiVariables = true;
				efiSysMountPoint = "/boot/efi";
			};
		};

		# Add support for ZFS.
		supportedFilesystems = [ "zfs" ];

		zfs = {
			forceImportRoot = false;
		};

		initrd = {
			availableKernelModules = [
				"nvme"
				"xhci_pci"
				"ahci"
				"usbhid"
				"usb_storage"
				"sd_mod"
			];

			kernelModules = [ ];

			# After the root pool has been decrypted…
			postMountCommands = lib.mkAfter ''
				# This snapshot was taken immediately after the root dataset was
				# created, capturing its initial state — nothingness. When the
				# system boots, it rolls back to this snapshot, cleaning up any
				# "dust" that had accumulated during the previous run.
				zfs rollback -r nuttyroot/root@void
			'';
		};

		kernelModules = [ "kvm-amd" ];
		extraModulePackages = [ ];
	};

	fileSystems = {
		"/" = {
			device = "nuttyroot/root";
			fsType = "zfs";
		};

		"/data" = {
			device = "nuttyroot/data";
			fsType = "zfs";
		};

		"/home" = {
			device = "nuttyroot/home";
			fsType = "zfs";
		};

		"/host" = {
			device = "nuttyroot/host";
			fsType = "zfs";

			# This directory contains the host SSH keys which are used to decrypt
			# age-encrypted secrets. Ensure it is mounted in the initial ramdisk.
			neededForBoot = true;
		};

		"/nix" = {
			device = "nuttyroot/nix";
			fsType = "zfs";
		};

		"/boot" = {
			device = "nuttyboot/boot";
			fsType = "zfs";
		};

		"/boot/efi" = {
			device = "/dev/disk/by-uuid/0BB5-D753";
			fsType = "vfat";
			options = [
				"fmask=0077"
				"dmask=0077"
			];
		};
	};

	swapDevices = [ ];

	hardware = {
		cpu = {
			amd = {
				updateMicrocode = lib.mkDefault config.hardware.enableRedistributableFirmware;
			};
		};
	};
}
