# Publishing LIPI to the internet

Everything is prepared. Two things ship: the **source repo** (to GitHub) and the
**release artifacts** (to GitHub Releases). Binaries are git-ignored — they belong
in Releases, not the repo.

## 1 · Push the source to GitHub

```sh
# In the repo root:
git remote add origin https://github.com/<you>/lipi-lang.git
git branch -M main
git push -u origin main
```

The moment this lands, GitHub Actions (`.github/workflows/ci.yml`) builds and runs
the test suite on Windows, Linux, and macOS — the first public proof it works
cross-platform.

## 2 · Cut a release (the downloadable artifacts)

The build produced two artifacts in `dist/` (git-ignored):

| File | What it is |
|------|-----------|
| `LIPI-Setup.exe` | Double-click installer (self-extracting; installs `lipi.exe` + adds it to PATH) |
| `lipi-0.2.0-windows-x64.zip` | Portable: `lipi.exe` + examples + the Handbook PDF |

Create a GitHub Release (tag `v0.2.0`) and upload both files:

```sh
# with the GitHub CLI:
gh release create v0.2.0 dist/LIPI-Setup.exe dist/lipi-0.2.0-windows-x64.zip \
  --title "LIPI 0.2.0" --notes "Devanagari-syntax language; bytecode VM; cross-platform stdlib."
```

Now the install commands in the README and the landing page resolve:
`irm .../install.ps1 | iex`, `winget install LIPI.LIPI`, `brew install <you>/tap/lipi`.

## 3 · Host the website + playground

`web/site/` is a self-contained static site with a **live** in-browser runner. Deploy
it anywhere static:

- **GitHub Pages:** enable Pages on the repo, serve `/web`. Free.
- **Netlify / Vercel / Cloudflare Pages:** publish directory `web/`, done.

See `docs/DEPLOY_PLAYGROUND.md` for the WASM build step and share-URL wiring.

## 4 · Update the placeholders

Replace `<you>` / `naraxcel` in these files with your GitHub handle:
`install.sh`, `install.ps1`, `packaging/homebrew/lipi.rb`,
`packaging/winget/LIPI.LIPI.yaml`, `web/site/index.html`, `README.md`.

## Checklist
- [ ] `git remote add origin … && git push -u origin main`
- [ ] `gh release create v0.2.0 dist/*.exe dist/*.zip`
- [ ] Enable GitHub Pages (or deploy `web/` to a host)
- [ ] Replace `<you>` placeholders with your handle
- [ ] (optional) Submit winget manifest / Homebrew tap / VSCode `.vsix`
