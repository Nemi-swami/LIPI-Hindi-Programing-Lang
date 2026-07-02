# LIPI — लिपि (VS Code Extension)

Syntax highlighting, snippets and editing support for the **LIPI programming language** — an Indian programming language using Devanagari script and Hindi/Sanskrit keywords.

**लिपि भाषा** के लिए VS Code एक्सटेंशन — सिंटैक्स हाइलाइटिंग, स्निपेट्स और एडिटिंग सपोर्ट।

## Features / सुविधाएँ

- Syntax highlighting for `.swami` files (also registers `.roman` and `.vani`)
- Keywords: `यदि`, `अन्यथा`, `विधि`, `वर्ग`, `विकल्प`, `जब तक`, `के लिए`, `कोशिश`/`पकड़ो`, …
- Word operators: `है`, `से अधिक`, `से कम`, `बराबर`, `और`, `या`, `नहीं`
- Both comment styles: `# …` and danda-delimited `। … ।`
- Strings with `{नाम}` interpolation and `{:.2}` format placeholders, plus `"""triple-quoted"""`
- ASCII and Devanagari digits (`५`, `१२.५`), `लाख` / `करोड़` suffixes
- Auto-indent after lines ending in `:`
- Snippets: type `विधि` / `vidhi`, `यदि` / `yadi`, `वर्ग` / `varg`, `कोशिश` / `koshish`, `विकल्प` / `vikalp`, …

## Install locally / स्थानीय इंस्टॉल

### Option A — copy the folder (no tools needed) / फ़ोल्डर कॉपी करें

1. Copy this whole `vscode-lipi` folder to:

   ```
   %USERPROFILE%\.vscode\extensions\lipi-lang.lipi-lang-0.1.0
   ```

   PowerShell:

   ```powershell
   Copy-Item -Recurse "D:\Projects\lipi-lang\vscode-lipi" "$env:USERPROFILE\.vscode\extensions\lipi-lang.lipi-lang-0.1.0"
   ```

   The destination folder name **must** follow `publisher.name-version` (here: `lipi-lang.lipi-lang-0.1.0`) so VS Code recognises it.

2. Restart VS Code (or run **Developer: Reload Window**).
3. Open any `.swami` file — highlighting is active.

### Option B — package with vsce / vsce से पैकेज बनाएँ

```powershell
npm install -g @vscode/vsce
cd D:\Projects\lipi-lang\vscode-lipi
vsce package          # produces lipi-lang-0.1.0.vsix
code --install-extension lipi-lang-0.1.0.vsix
```

### Uninstall / हटाएँ

Option A: delete the folder from `%USERPROFILE%\.vscode\extensions\`.
Option B: `code --uninstall-extension lipi-lang.lipi-lang`

## About LIPI

LIPI is written in pure Rust with a bytecode VM (LVM). Source files use the `.swami` extension; Roman-QWERTY input uses `.roman` and full phonetic input uses `.vani`.

```lipi
। एक छोटा उदाहरण ।
विधि जोड़ो(अ, ब):
    फल अ + ब

बताओ "योग: {परिणाम}"
```
