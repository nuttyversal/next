local conform = require("conform")

conform.setup({
	formatters = {
		nixfmtty = {
			command = "nixfmtty",
			args = { "$FILENAME" },

			-- Modifies in-place.
			stdin = false,
		},
	},

	formatters_by_ft = {
		javascript = { "prettier" },
		javascriptreact = { "prettier" },
		typescript = { "prettier" },
		typescriptreact = { "prettier" },
		json = { "prettier" },
		lua = { "stylua" },
		nix = { "nixfmtty" },
	},

	format_on_save = {
		timeout_ms = 500,
		lsp_format = "fallback",
	},
})
