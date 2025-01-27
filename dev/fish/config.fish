if status is-interactive
	set config_dir (dirname (status -f))
	source $config_dir/config/abbreviations.fish
	source $config_dir/config/greeting.fish
	source $config_dir/config/prompt.fish
	source $config_dir/config/variables.fish
end
