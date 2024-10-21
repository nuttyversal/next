local lazy_path = vim.fn.stdpath("data") .. "/lua/lazy.lua"

if not (vim.uv or vim.loop).fs_stat(lazy_path) then
	local clone_location = vim.fn.system({
		"git",
		"clone",
		"--filter=blob:none",
		"--branch=stable",
		"https://github.com/folke/lazy.nvim.git",
		lazy_path,
	})

	if vim.v.shell_error ~= 0 then
		vim.api.nvim_echo({
			{ "Failed to clone lazy.nvim:\n", "ErrorMsg" },
			{ clone_location, "WarningMsg" },
			{ "\nPress any key to exit…" },
		}, true, {})

		-- Waiting for key…
		vim.fn.getchar()

		os.exit(1)
	end
end

vim.opt.rtp:prepend(lazy_path)

require("lazy").setup("plugins")
