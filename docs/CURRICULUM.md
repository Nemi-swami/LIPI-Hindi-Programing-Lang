# LIPI — a Hindi-first introductory CS curriculum

A course outline for teaching programming in Hindi using LIPI, structured to align
with CBSE/state-board introductory computer-science goals. Zero-install: every
lesson runs in LIPI Studio (the browser playground) or with a single `lipi`
binary. Each unit lists concepts, LIPI features, and a runnable exercise.

> This is a curriculum **outline** for educators to adapt — not an accredited
> syllabus. Real board adoption is a policy process, not a code artifact.

## Unit 0 — Setup (1 lesson)
- Open LIPI Studio in a browser, or install the CLI.
- First program: `बताओ "नमस्ते दुनिया"`. Run it. See output.

## Unit 1 — Values & variables (2 lessons)
- Concepts: data, assignment, numbers vs text.
- LIPI: `है`, `बताओ`, `लिखो`, Devanagari digits (`५`, `१२`), arithmetic, `//`.
- Exercise: compute the area of a rectangle from two variables.

## Unit 2 — Decisions (2 lessons)
- Concepts: conditions, comparison, boolean logic.
- LIPI: `यदि`/`अन्यथा यदि`/`अन्यथा`, `से अधिक`/`से कम`/`बराबर`, `और`/`या`/`नहीं`.
- Exercise: grade calculator (marks → grade).

## Unit 3 — Repetition (3 lessons)
- Concepts: loops, iteration, accumulation.
- LIPI: `बार करो`, `के लिए … में`, `जब तक`, `बंद करो`, `अगला`.
- Exercise: multiplication table; sum of first N numbers.

## Unit 4 — Collections (3 lessons)
- Concepts: lists, dictionaries, indexing, iteration over data.
- LIPI: `[…]`, `{…}`, index/slice, `.जोड़ो()`, `.लम्बाई()`, dict keys.
- Exercise: student marksheet stored as a list of dicts.

## Unit 5 — Functions (3 lessons)
- Concepts: abstraction, parameters, return values, reuse.
- LIPI: `विधि`, `फल`, default params, keyword args.
- Exercise: a reusable `जीएसटी_जोड़ो(राशि, दर)` function.

## Unit 6 — Text & files (2 lessons)
- Concepts: strings, formatting, reading/writing files.
- LIPI: string methods, `स्वरूप()`, `संचिका_सामग्री`/`संचिका_लिखो`, CSV module.
- Exercise: read a CSV of names+marks, write a formatted report.

## Unit 7 — Real Indian-context project (3 lessons)
- Concepts: putting it together on a meaningful problem.
- LIPI: `आयात भारत.पहचान` (Aadhaar), `भारत.संख्या` (GST/rupees), `भारत.समय`.
- Capstone options:
  - Aadhaar batch validator (read a list, flag invalid ones).
  - GST invoice generator (items → totals → formatted bill).
  - Attendance/marks tracker with a CSV report.

## Assessment
- Per-unit runnable exercises (auto-checkable with `lipi test` + `परीक्षण` blocks).
- Capstone project graded on correctness + clarity.

## Why LIPI for this audience
- No English-keyword tax: students read code in the language they think in.
- Error messages are in Hindi.
- Indian-context stdlib means the capstone projects are motivating and real.
- Browser playground → no lab setup, works on any school computer.
