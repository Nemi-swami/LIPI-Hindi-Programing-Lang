# Changelog

## 0.2.0 — 2026-06-29

Phase 17 completion update.

- Added keywords to the grammar: `उत्पन्न` (yield / generators), `फेंको` (throw),
  `साथ` / `के_रूप_में` (context managers), `साझा विधि` (static methods),
  `सार वर्ग` (abstract classes), `अभिलेख` (records/dataclasses), and the `शून्य`
  (nil) constant.
- Tracks new language features and stdlib modules shipped in Phase 17
  (generators, functools, bignum, sockets, zip, SQL, NFC normalization).

### Publishing to the Marketplace (maintainer action)

The `.vsix` is built (`lipi-lang-0.2.0.vsix`). To publish you need a
Visual Studio Marketplace **publisher account** and a **Personal Access Token**:

```
npm i -g @vscode/vsce
vsce login lipi-lang          # paste your Azure DevOps PAT
vsce publish                  # from vscode-lipi/  (or: vsce publish 0.2.0)
```

`vsce package` (no auth) just rebuilds the `.vsix` for local install via
`code --install-extension lipi-lang-0.2.0.vsix`.

## 0.1.0 — 2026-06-12

Initial release.

- TextMate grammar for LIPI (`source.lipi`): keywords, word operators, constants,
  ASCII + Devanagari numbers with लाख/करोड़ suffixes, `#` and `।…।` comments,
  strings with interpolation/format placeholders, triple-quoted strings,
  function-call and `यह` highlighting, symbol operators (incl. `//` floor-divide).
- Language registration for `.swami`, `.roman`, `.vani`.
- Language configuration: brackets, auto-closing pairs, indent after `:`.
- Snippets: विधि, यदि/अन्यथा, जब तक, के लिए, बार करो, वर्ग+बनाओ, कोशिश/पकड़ो, विकल्प+मिलाओ, लाम्डा, बताओ.
