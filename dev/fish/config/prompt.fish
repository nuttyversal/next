function fish_prompt
	# Did the last command fail?
	if test $status -ne 0
		set_color red
		printf 'Oof… '
		set_color normal
	end

	# Sir Shelly, where are we?
	if test (pwd) = /
		printf 'Behold, the '
		set_color red
		printf '/'
		set_color normal
		printf ' of all files on '
	else if test (pwd) = ~
		printf 'Home, sweet '
		set_color yellow
		printf '~'
		set_color normal
		printf ' in '
	else if string match -q "*/Downloads" (pwd)
		printf 'Fresh plunder awaits in '
		set_color yellow
		printf (prompt_pwd --full-length-dirs=2)
		set_color normal
		printf ' on '
	else
		printf 'We are in '
		if string match -q "*/Nuttyverse" (pwd)
			printf 'the '
		end
		set_color yellow
		printf (prompt_pwd --full-length-dirs=2)
		set_color normal
		printf ' on '
	end

	set_color yellow
	printf (hostname)
	set_color normal
	echo "."

	# Sir Shelly!
	set special_prompts \
		'Yes, my lord? ' \
		'How may I serve? ' \
		'Command me, sire! ' \
		'What is thy bidding? ' \
		'Awaiting orders… '

	set special_index (random 1 (count $special_prompts))

	if not set -q __fish_prompt_first_time
		set -g __fish_prompt_first_time 1

		# Beginner's luck.
		set_color yellow
		printf $special_prompts[$special_index]
	else
		set_color yellow

		# Roll that D20. Need an 18 or higher.
		if test (random 1 100) -le 15
			printf $special_prompts[$special_index]
		else
			printf 'λ: '
		end
	end

	set_color normal
end

function postexec_test --on-event fish_postexec
	if test $status -ne 0
		# Display an error banner for failed commands.
		set_color --background red --bold black
		printf "[ERROR] Command failed with status code ($status)"
		set_color normal

		# Defer printing the line break until after the color
		# is set back to normal so that the red doesn't bleed
		# into the next line when the terminal scrolls.
		echo
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
