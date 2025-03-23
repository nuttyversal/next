local blink = require("blink.cmp")

blink.setup({
	keymap = "default",
	nerd_font_variant = "mono",

	-- Show function signature hints.
	-- Useful for referencing parameter types.
	trigger = {
		signature_help = {
			enabled = true,
			show_on_insert_on_trigger_character = true,
		},
	},

	windows = {
		autocomplete = {
			border = "single",
		},

		documentation = {
			border = "single",
			auto_show = true,
			auto_show_delay_ms = 500,
			update_delay_ms = 50,
		},

		signature_help = {
			border = "single",
		},
	},
})
