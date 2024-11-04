#!/usr/bin/env -S just --justfile

set export
set shell := ["fish", "--no-config", "--command"]

@ci *COMMAND:
	echo "[INFO] Entering the Nuttyverse CI…"
	nix develop \
		--no-update-lock-file \
		--ignore-environment \
		--command {{COMMAND}}

@develop:
	echo "[INFO] Entering the Nuttyverse shell…"
	nix develop --command fish

@build:
	echo "[INFO] Building the Nuttyverse…"
	fish --no-config ./ci/just-recursively.fish build

@build-root:
	nix build
	nixfmtty flake.lock

@test:
	echo "[INFO] Testing the Nuttyverse…"
	fish --no-config ./ci/just-recursively.fish test

@deploy:
	echo "[INFO] Deploying the Nuttyverse…"
	fish --no-config ./ci/just-recursively.fish deploy

@clean:
	echo "[INFO] Cleaning the Nuttyverse…"
	rm --force --verbose result
	fish --no-config ./ci/just-recursively.fish clean

@update:
	echo "[INFO] Updating the Nuttyverse…"
	nix flake update
	nixfmtty flake.lock
	fish --no-config ./ci/just-recursively.fish update
