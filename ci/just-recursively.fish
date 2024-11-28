#!/usr/bin/env fish --no-config

set justfiles (find . -name justfile)
set justargs $argv

for justfile in $justfiles
	# Ignore the root justfile.
	# Prevents infinite recursion.
	if test $justfile = "./justfile"
		continue
	end

	# Does the recipe exist?
	just \
		--justfile=$justfile \
		--working-directory=(dirname $justfile) \
		--list \
			| rg "^\s*$justargs[1]\$" > /dev/null

	if test $status -ne 0
		continue
	end

	# Then, let 'em cook!
	just \
		--justfile=$justfile \
		--working-directory=(dirname $justfile) \
		ci \
			just $justargs

	# Fail fast if a command fails.
	if test $status -ne 0
		exit $status
	end
end
