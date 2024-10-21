#!/usr/bin/env -S just --justfile

set export
set shell := ["nix", "develop", "-c", "fish", "-c"]

@develop:
	echo "[INFO] Entering the Nuttyverse shell…"
	nix develop -c fish

@build:
	echo "[INFO] Building the Nuttyverse…"
	nix build
	nixfmtty flake.lock
	./ci/just-recursively.fish build

@test:
	echo "[INFO] Testing the Nuttyverse…"
	./ci/just-recursively.fish test

@deploy:
	echo "[INFO] Deploying the Nuttyverse…"
	./ci/just-recursively.fish deploy

@clean:
	echo "[INFO] Cleaning the Nuttyverse…"
	rm -fv result
	./ci/just-recursively.fish clean

@update:
	echo "[INFO] Updating the Nuttyverse…"
	nix flake update
	nixfmtty flake.lock
	./ci/just-recursively.fish update
