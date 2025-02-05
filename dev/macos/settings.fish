if test (uname) = "Darwin"
	# Configure the Dock.
	defaults write com.apple.dock "autohide" -bool "true"
	defaults write com.apple.dock "autohide-delay" -float "0.0"
	defaults write com.apple.dock "magnification" -bool "true"
	defaults write com.apple.dock "tilesize" -int "48"
	defaults write com.apple.dock "largesize" -int "72"

	# Stop expanding double spaces into periods.
	# It conflicts with my <Leader><Leader> mapping.
	defaults write NSGlobalDomain "NSAutomaticPeriodSubstitutionEnabled" -int "0"

	# Minimize input lag by increasing ARR and decreasing DAS.
	defaults write NSGlobalDomain "ApplePressAndHoldEnabled" -bool "false"
	defaults write NSGlobalDomain "InitialKeyRepeat" -int "10"
	defaults write NSGlobalDomain "KeyRepeat" -int "1"
end
