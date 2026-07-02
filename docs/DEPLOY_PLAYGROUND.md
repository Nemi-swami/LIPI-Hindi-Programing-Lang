# Hosting the LIPI playground + share-URLs

The runtime already compiles to `wasm32-unknown-unknown` and runs fully in the
browser (`web/`, `web/studio/`, built with `wasm-pack build --target web
--out-dir web/pkg --features wasm`). Putting it online is a **static-host deploy** —
no server, no backend. This is the highest-visibility, lowest-effort growth lever.

## Deploy (any static host)

The `web/` directory after a WASM build is a self-contained static site. Deploy it
to any static host:

- **GitHub Pages:** push `web/` (with `web/pkg/`) to a `gh-pages` branch, or point
  Pages at `/web`. Free, zero-config.
- **Netlify / Vercel / Cloudflare Pages:** set the publish directory to `web/` and
  the build command to the `wasm-pack` line above. Drag-and-drop also works.
- **Any CDN/bucket:** it's just static files — `.html`, `.js`, `.wasm`.

The one requirement: the `.wasm` file must be served with
`Content-Type: application/wasm` (all the hosts above do this automatically).

## Share-URLs ("try it, share a link")

The pattern that makes a playground spread: encode the editor's source in the URL so
a link reproduces the program. Client-side only:

- **Share:** base64-encode the editor content and put it in the URL fragment —
  `https://<host>/#code=<base64>`. The fragment never hits a server, so no backend
  and no storage are needed.
- **Load:** on page load, if `location.hash` contains `code=`, base64-decode it into
  the editor and (optionally) auto-run.

Sketch to wire into the playground page:

```js
// Share button:
const enc = btoa(unescape(encodeURIComponent(editor.getValue())));
navigator.clipboard.writeText(location.origin + location.pathname + '#code=' + enc);

// On load:
const m = /#code=([^&]+)/.exec(location.hash);
if (m) editor.setValue(decodeURIComponent(escape(atob(m[1]))));
```

Use the URL fragment (`#`), not a query param, so the code stays client-side and
links work on a pure static host. For very long programs, gzip before base64
(`CompressionStream`) to keep URLs short.

## What blocks "done"
- A host + (optionally) a domain — an operator action, not repo work.
- Running the `wasm-pack` build in CI and publishing `web/` on each release (a
  GitHub Pages deploy job is ~15 lines; add it once the repo has a remote).
