# lipi.dev — package registry design

`lipi pkg` already installs dependencies **git-natively** (`lipi pkg add <name>
<git-url>#<tag>` → clone into `lipi_modules/`). That covers 90% of what a registry
does without needing a server. A hosted registry adds: discovery (search),
canonical names (not full URLs), and version resolution. This doc specifies it so
it can be built when the audience justifies a server — the code artifact here is the
**design + the client-side flow**, since the hosting itself needs a domain and a
host account (a real-world action, not a repo change).

## Two-layer model (ship the first, add the second later)

**Layer 1 — git-native (works today, no server).**
- `lipi pkg add ganita https://github.com/you/lipi-ganita.git#v1.0`
- `lipi pkg install`
- Distribution = push a repo. This is the recommended path right now.

**Layer 2 — hosted index (`lipi.dev`, when there's demand).**
A registry is just a name→git-URL index plus metadata. Minimal shape:

```
GET  https://lipi.dev/api/pkg/<name>            → { name, versions: [...], repo }
GET  https://lipi.dev/api/pkg/<name>/<version>  → { repo, tag, sha, checksum }
POST https://lipi.dev/api/publish               → registers name → repo (auth'd)
GET  https://lipi.dev/api/search?q=<term>       → [ {name, description}, ... ]
```

The registry stores **no package bytes** — only the mapping to a git tag + a
content checksum. `lipi pkg add ganita` resolves `ganita` → repo+tag via the index,
then clones exactly as Layer 1 does. This keeps the server tiny (a key-value store +
a JSON API) and the trust model simple (packages live in their authors' repos).

## `lipi publish` flow (Layer 2 client side)

1. Read `lipi.toml` for `name`, `version`, `repository`.
2. Verify the working tree is clean and tagged `v<version>`.
3. `POST /api/publish` with `{name, version, repo, tag, checksum}` + an auth token.
4. Registry records the mapping; `lipi pkg add <name>` now resolves it.

## Security / integrity
- Names are first-come, tied to the publisher's auth identity.
- Every version pins a git SHA + a checksum; `lipi pkg install` verifies the
  checksum after cloning. A moved/rewritten tag fails verification.
- No arbitrary code runs at install time (LIPI packages are source, imported at
  compile time) — the install step only clones and checksums.

## What blocks Layer 2
- A domain (`lipi.dev`) and a host for the JSON API + KV store.
- An auth story (GitHub OAuth is the natural fit).
Both are operator actions, not repository work. The `lipi pkg` client is already
structured (see `src/pkg.rs`) so adding a `--registry <url>` resolver is a localized
change when the host exists.
