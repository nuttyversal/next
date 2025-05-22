function envsource
	for line in (cat $argv | grep -v -e '^#' -e '^$')
		set item (string split -m 1 '=' $line)
		set -gx $item[1] $item[2]
		echo "Exported environment variable: $item[1]."
	end
end
