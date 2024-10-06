#!/usr/bin/env -S just --justfile

set shell := ["fish", "-c"]

develop:
	@echo "[INFO] Entering the Nuttyverse shell…"
	@nix develop --command fish

build:
	@echo "[INFO] Building the Nuttyverse…"
	@nix build
	@nixfmtty flake.lock

test:
	@echo "[INFO] Testing the Nuttyverse…"
	@./ci/just-recursively.fish test

deploy:
	@echo "[INFO] Deploying the Nuttyverse…"
	@./ci/just-recursively.fish deploy

update:
	@echo "[INFO] Updating the Nuttyverse…"
	@nix flake update
	@nixfmtty flake.lock
	@./ci/just-recursively.fish update
