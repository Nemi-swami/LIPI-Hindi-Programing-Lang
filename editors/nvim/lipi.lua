-- LIPI support for Neovim — filetype detection, comment/indent, and LSP client.
--
-- Install: copy this file somewhere on your runtimepath, or drop it in
--   ~/.config/nvim/lua/lipi.lua  and add  require('lipi').setup()  to init.lua.
-- Requires the `lipi` binary on PATH (provides `lipi lsp` over stdio).

local M = {}

function M.setup(opts)
  opts = opts or {}
  local cmd = opts.cmd or { 'lipi', 'lsp' }

  -- 1. Filetype detection for LIPI source extensions.
  vim.filetype.add({
    extension = {
      swami = 'lipi',
      roman = 'lipi',
      vani = 'lipi',
    },
  })

  -- 2. Per-buffer options + LSP attach when a LIPI file opens.
  vim.api.nvim_create_autocmd('FileType', {
    pattern = 'lipi',
    callback = function(args)
      vim.bo[args.buf].commentstring = '# %s'
      vim.bo[args.buf].expandtab = true
      vim.bo[args.buf].shiftwidth = 4
      vim.bo[args.buf].tabstop = 4

      -- Start (or reuse) the LIPI language server for this buffer.
      vim.lsp.start({
        name = 'lipi-lsp',
        cmd = cmd,
        root_dir = vim.fs.dirname(vim.fs.find({ 'lipi.toml', '.git' }, {
          upward = true,
          path = vim.api.nvim_buf_get_name(args.buf),
        })[1]) or vim.fn.getcwd(),
      })
    end,
  })
end

return M
