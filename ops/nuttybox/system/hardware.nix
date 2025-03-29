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
			# The root pool is encrypted, so when the system boots, it must be
			# decrypted with an encryption passphrase before it can be imported.
			requestEncryptionCredentials = true;
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

		# Disable Active State Power Management.
		# It is making the PCIe link unstable,
		# which disconnects the network device.
		kernelParams = [ "pcie_aspm=off" ];

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
			device = "/dev/disk/by-uuid/E31B-54FD";
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

		system76 = {
			enableAll = true;
		};
	};
}
