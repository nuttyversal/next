#!/usr/bin/env fish

function check
	set file $argv[1]
	set temp_file (mktemp)
	set diff_file (mktemp)

	# Copy file into workspace.
	cp $file $temp_file

	# Is the file indented with tabs?
	unexpand --tabs=2 --first-only $temp_file > $diff_file
	diff $temp_file $diff_file > /dev/null
	set diff_status $status

	# Is the file formatted with nixfmt?
	expand --tabs=2 $temp_file | sponge $temp_file
	nixfmt -c $temp_file
	set fmt_status $status

	# Clean up workspace.
	rm $temp_file
	rm $diff_file

	if test $diff_status -eq 0 -a $fmt_status -eq 0
		ok $file "looks good"
		return 0
	else
		fail $file "needs formatting"
		return 1
	end
end

function format
	set file $argv[1]

	# Format the file with nixfmt.
	nixfmt $file 2> /dev/null

	# [HACK] Format again to ensure idempotent formatting.
	# Prevents multi-string literals from being indented
	# an extra level whenever nixfmtty is run.
	nixfmt $file 2> /dev/null

	if test $status -ne 0
		fail $file "could not be formatted"
		return 1
	end

	# Convert spaces to tabs.
	unexpand --tabs=2 $file | sponge $file

	ok $file "has been formatted"
	return 0
end

function ok
	set file $argv[1]
	set message $argv[2]
	echo "[OK] ━━━ ❬$file❭ $message."
end

function fail
	set file $argv[1]
	set message $argv[2]
	echo "[FAIL] ━ ❬$file❭ $message." >&2
end

argparse 'c/check' -- $argv
or exit

if set -ql _flag_check
	set mode "check"
else
	set mode "format"
end

if test (count $argv) -eq 0
	echo "Usage: nixfmtty [OPTIONS] FILES…"
	echo "Options:"
	echo "   -c, --check | Check if the files are formatted correctly."
	exit 1
end

set exit_code 0

for file in $argv
	if test -f $file
		if test $mode = "check"
			check $file

			if test $status -ne 0
				set exit_code 1
			end
		else
			format $file

			if test $status -ne 0
				set exit_code 1
			end
		end
	else
		fail $file "is not a file"
	end
end

exit $exit_code
