# LIPI packages — a starter library collection

Small, dependency-free LIPI libraries you can `आयात` into your programs. This is a
**seed collection** meant to grow toward a full ecosystem — not 50 production
libraries yet. Each file is plain LIPI source; import it by path.

```lipi
आयात "packages/ganit_sahayak.swami"
बताओ गसद(48, 36)      # 12
बताओ अभाज्य_है(97)     # सत्य
```

| Package | Functions |
|---------|-----------|
| `paath.swami` | दोहराओ, शब्द_गिनती, उलटा_शब्द, पैड_बाएं, है_उपसर्ग |
| `ganit_sahayak.swami` | गसद (gcd), लसद (lcm), अभाज्य_है (is-prime), सीमित (clamp), क्रमगुणित |
| `soochi.swami` | योग, अधिकतम, न्यूनतम, अद्वितीय (unique), माध्य |
| `satyapan.swami` | खाली_नहीं, सीमा_में, ईमेल_सा, केवल_अंक, लंबाई_ठीक |
| `rupantar.swami` | temperature / distance / mass / angle conversions |
| `aankde.swami` | माध्य, माध्यिका, प्रसरण, मानक_विचलन (self-contained stats) |
| `vitt.swami` | जीएसटी_जोड़ो, साधारण_ब्याज, चक्रवृद्धि, ईएमआई |
| `format.swami` | दो_दशमलव, प्रतिशत, भारतीय_अंक, रुपये, शीर्षक_रेखा |

~40 functions across 8 packages. Contributions welcome — a library is just a
`.swami` file exporting `विधि`s. To publish one, push it to a git repo and add it
with `lipi pkg add <name> <git-url>` (see `docs/REGISTRY.md`).
