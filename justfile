#!/usr/bin/env -S just --justfile

set shell := ["fish", "-c"]

develop:
	nix develop --command fish

build:
	nix build
	nixfmtty flake.lock

test:
	./ci/just-recursively.fish test

update:
	nix flake update
	nixfmtty flake.lock
	./ci/just-recursively.fish update
