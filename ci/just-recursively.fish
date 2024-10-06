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
		$justargs
end
