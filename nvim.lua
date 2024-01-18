-- This is a script used to debug `harper-ls` in NeoVim.

vim.lsp.start({
  name = "example",
  cmd = { "harper-ls" },
  root_dir = "."
})
