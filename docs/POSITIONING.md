# LIPI Positioning — the one thing to own

Languages don't win on feature count; they win by owning **one domain** so
completely that, in that domain, no alternative is considered. Python owns
ML/scripting, JavaScript owns the browser, Rust owns memory-safe systems. LIPI
must pick its wedge and dominate it before trying to be general-purpose.

## The wedge: Hindi-native programming + Indian-context computing

LIPI is the only serious language whose keywords, errors, and standard library are
**Devanagari-native**, and whose stdlib already speaks Indian domains (Aadhaar
verification, UPI, GST, IST datetime, big-integer math, the mission-critical suite).
Two audiences no other language serves:

1. **First-time programmers who think in Hindi.** For a student who hasn't mastered
   English, every English-keyword language adds a translation tax on top of learning
   to code. LIPI removes it. This is the single largest untapped programmer
   population in the world.

2. **Indian-context back-office/fintech/govtech scripts.** Aadhaar/PAN/IFSC
   validation, UPI flows, GST math, Indian number formatting (lakh/crore), and IST
   handling are first-class — not third-party packages. For that work, LIPI is the
   shortest path from problem to correct code.

## What to say (and not say)

- **Say:** "Write correct code in the language you think in. Batteries included for
  Indian-context computing." Show a 5-line Aadhaar+UPI+GST script that would be 50
  lines elsewhere.
- **Don't say:** "faster than Python / better than Rust." It's a bytecode VM; it
  isn't, yet. Competing on raw speed loses. Compete on *accessibility* and
  *domain fit*, where it genuinely wins.

## The three concrete beachheads (in priority order)

1. **School/college CS in Hindi.** A CBSE-aligned curriculum (see
   `docs/CURRICULUM.md`) + the browser playground = zero-install classroom use. This
   is the defensible, high-volume path.
2. **Indian-context scripting.** Publish 20 real worked examples: GST invoice
   generator, UPI reconciliation, Aadhaar batch validator, PDF/CSV report tools.
3. **Regional-language app logic.** LIPI compiles to WASM — pair it with a thin UI
   for Hindi-first apps.

## How each tier serves the wedge

- **Distribution (Tier 1)** makes the wedge reachable: cross-platform + one-command
  install + a shareable browser playground are what let a teacher or student start
  in 30 seconds.
- **Ecosystem (Tier 3)** deepens it: the config/data formats, HTTPS, and FFI are
  what turn "teaching language" into "gets real work done."
- **Community/education (Tier 4)** is the actual moat: Hindi-first docs, tutorials,
  and curriculum are things a general-purpose language will never bother to build
  for this audience.

The strategic error to avoid: spreading effort trying to be a better general-purpose
language. Own the Hindi-native + Indian-context niche first. Breadth comes after
depth, not instead of it.
