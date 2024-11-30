local avante = require("avante")

avante.setup({
	provider = "claude",
	auto_suggestions_provider = "copilot",

	claude = {
		endpoint = "https://api.anthropic.com",
		model = "claude-3-5-sonnet-20241022",
		temperature = 0,
		max_tokens = 4096,
	},
})
