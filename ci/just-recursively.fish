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
	if test "$justargs[1]" = "update"
		# Cloning git repos with submodules does not seem to work under the
		# `just ci` context. I am not concerned about hermetic environments
		# when fetching flakes externally.
		just \
			--justfile=$justfile \
			--working-directory=(dirname $justfile) \
			$justargs
	else
		just \
			--justfile=$justfile \
			--working-directory=(dirname $justfile) \
			ci \
				just $justargs
	end

	# Fail fast if a command fails.
	if test $status -ne 0
		exit $status
	end
end
