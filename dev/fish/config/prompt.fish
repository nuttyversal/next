function fish_prompt
	echo [(hostname)] (prompt_pwd --full-length-dirs=2)
	printf 'λ. '
end

function postexec_test --on-event fish_postexec
	if test $status -ne 0
		# Output an error banner for failed commands.
		set_color -b red black
		echo "[ERROR] Command failed with status code ($status)"
		set_color normal
	end

	if not test $argv = 'clear'
		# Append a line break at the end of the command output.
		# It makes it easier to read the scrollback buffer.
		#
		# > $ hostname
		# > nuttybook
		# >
		# > $ whoami
		# > nutty
		echo
	end
end

function on_cancel --on-event fish_cancel
	# Append a line break after an cancelled command.
	# Commands can be cancelled with Ctrl + c.
	#
	# > $ hostna^C
	# >
	# > $ hostname
	# > nuttybook
	echo
end
