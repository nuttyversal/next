#!/usr/bin/env fish

set justfiles (ls **/justfile)
set justargs $argv

for justfile in $justfiles
	# Ignore the root justfile.
	# Prevents infinite recursion.
	if test $justfile = "justfile"
		continue
	end

	just \
		--justfile=$justfile \
		--working-directory=(dirname $justfile) \
		--shell "nix" \
		--shell-arg "develop" \
		--shell-arg "-u" \
		--shell-arg "PATH" \
		--shell-arg "-c" \
		--shell-arg "fish" \
		--shell-arg "-c" \
		$justargs

	# Fail fast if a command fails.
	if test $status -ne 0
		exit $status
	end
end
