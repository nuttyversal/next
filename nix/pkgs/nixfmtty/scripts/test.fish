#!/usr/bin/env fish --no-config

function log
	set result $argv[1]
	set description $argv[2]

	if test $result -eq 0
		echo "[OK] ━━━ $description"
		return 0
	else
		echo "[FAIL] ━ $description"
		return 1
	end
end

set script_dir (dirname (status -f))
set fixtures_dir "$script_dir/../fixtures"
set bin_dir "$script_dir/../result/bin"

if not test -d $bin_dir
	echo "Run `nix build` before running tests."
	exit 1
end

$bin_dir/nixfmtty > /dev/null 2>&1
test $status -eq 1
log $status "no files were provided"

$bin_dir/nixfmtty --check $fixtures_dir/formatted.nix > /dev/null 2>&1
test $status -eq 0
log $status "check formatted.nix"

$bin_dir/nixfmtty --check $fixtures_dir/unformatted.nix > /dev/null 2>&1
test $status -eq 1
log $status "check unformatted.nix"

$bin_dir/nixfmtty --check $fixtures_dir/*.nix > /dev/null 2>&1
test $status -eq 1
log $status "check *.nix"

set unformatted (mktemp --suffix .nix)
cp $fixtures_dir/unformatted.nix $unformatted
$bin_dir/nixfmtty $unformatted > /dev/null 2>&1
diff $unformatted $fixtures_dir/formatted.nix > /dev/null 2>&1
test $status -eq 0
log $status "format unformatted.nix"
rm $unformatted

set formatted (mktemp --suffix .nix)
cp $fixtures_dir/formatted.nix $formatted
$bin_dir/nixfmtty $formatted > /dev/null 2>&1
diff $formatted $fixtures_dir/formatted.nix > /dev/null 2>&1
test $status -eq 0
log $status "format formatted.nix"
rm $formatted

set formatted (mktemp --suffix .nix)
set unformatted (mktemp --suffix .nix)
cp $fixtures_dir/formatted.nix $formatted
cp $fixtures_dir/unformatted.nix $unformatted
$bin_dir/nixfmtty $formatted $unformatted > /dev/null 2>&1
diff $formatted $fixtures_dir/formatted.nix; and \
	diff $unformatted $fixtures_dir/formatted.nix > /dev/null 2>&1
test $status -eq 0
log $status "format *.nix"
