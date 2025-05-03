local blink = require("blink.cmp")

blink.setup({
	keymap = {
		preset = "default",
	},

	appearance = {
		nerd_font_variant = "mono",
	},

	completion = {
		documentation = {
			auto_show = true,
		},

		trigger = {
			show_on_insert_on_trigger_character = true,
		},
	},

	signature = {
		enabled = true,
	},

	fuzzy = {
		implementation = "lua",
	},

	sources = {
		default = {
			"lsp",
			"buffer",
			"snippets",
			"path",
		},

		per_filetype = {
			codecompanion = { "codecompanion" },
		},
	},
})
