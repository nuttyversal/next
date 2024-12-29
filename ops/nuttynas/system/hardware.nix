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
			# age-encrypted secrets and the ZFS keys for mounting encrypted datasets.
			# Ensure it is mounted in the initial ramdisk.
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

	services = {
		zfs = {
			autoSnapshot = {
				enable = true;

				# 15-minute snapshots, keeping last 4; 1 hour of quick recovery.
				frequent = 4;

				# Hourly snapshots, keeping last 24; 1 day of hourly points.
				hourly = 24;

				# Daily snapshots, keeping last 7; 1 week of daily points.
				daily = 7;

				# Weekly snapshots, keeping last 4; 1 month of weekly points.
				weekly = 4;

				# Monthly snapshots, keeping last 12; 1 year of monthly points.
				monthly = 12;
			};

			# Check & correct data corruption.
			autoScrub = {
				enable = true;
				interval = "weekly";
			};
		};
	};
}
