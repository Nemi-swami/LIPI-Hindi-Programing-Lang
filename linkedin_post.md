# LinkedIn Post — LIPI Programming Language

---

## 🇮🇳 I built a programming language where you code in Hindi — and it honors 1,500 years of Indian mathematics

---

For the past few weeks I've been building **LIPI** — a programming language that uses Devanagari script and Hindi/Sanskrit keywords, compiled to a custom bytecode VM written entirely in pure Rust.

Phase 14 just shipped, and I want to share what makes it special.

---

### The language looks like this:

```
आयात भारत.गणित

बताओ स्वरूप("ज्या(30°) = {:.4}", ज्या(पाई / 6))
बताओ स्वरूप("C(10,3) = {}", संयोजन(10, 3))
बताओ स्वरूप("विरहांक(10) = {}", विरहांक(10))
बताओ स्वरूप("ब्रह्मगुप्त सूत्र = {:.4}", ब्रह्मगुप्त_क्षेत्र(3, 4, 5, 6))
```

Output:
```
ज्या(30°) = 0.5000
C(10,3) = 120
विरहांक(10) = 55
ब्रह्मगुप्त सूत्र = 18.9737
```

---

### The history behind `भारत.गणित` (Bharat.Math)

Every function in the module is named after its original Indian source:

**ज्या (jyā)** — Āryabhaṭīya, 499 CE
The Sanskrit word for the chord of an arc. Arabic translators phonetically copied it as *jiba*, which was later mistranslated from Arabic as *sinus* — giving us the word **"sine"**. The origin of trigonometry lives in this one word.

**विरहांक (Virahāṅka)** — Chandahśāstra, ~600 CE
What the world calls the "Fibonacci sequence" was described by Virahāṅka ~600 years before Fibonacci (1202 CE). He was analyzing Sanskrit poetic meters. LIPI names the function after its actual inventor.

**संयोजन / क्रमचय** — Bhāskara II, Līlāvatī, 1150 CE
Combinations C(n,r) and permutations P(n,r) from the *Līlāvatī* — a mathematics textbook written as a poem, addressed to a girl named Leelavati. Bhāskara computed these 400+ years before European combinatorics.

**ब्रह्मगुप्त_क्षेत्र** — Brāhmasphuṭasiddhānta, 628 CE
Brahmagupta's formula for the area of a cyclic quadrilateral — and the first written treatment of zero as a number.

---

### What LIPI actually is (technically)

- **Pure Rust** — lexer, parser, bytecode compiler, stack VM, stdlib — zero external crates
- **Devanagari-first** — all keywords are Hindi/Sanskrit (`बताओ`, `यदि`, `विधि`, `फल`)
- **Devanagari digit literals** — `क है ५` is valid assignment; `i के लिए १० में` loops 0–9
- **Full language features** — classes, closures, lambdas, HOFs, try/catch, inheritance, varargs, bitwise ops
- **`भारत` stdlib** — 5 modules covering Aadhaar verification, UPI payments, GST, Indian number formatting, ancient math
- **WASM build** — runs in the browser via a playground

---

### Roman QWERTY input (just shipped)

Don't have a Devanagari keyboard? No problem. You can now write:

```
batao "namaste duniya!"
ka hai 10
yadi ka se adhik 5:
    batao "bada hai"
aayat bharat.ganit
batao swaroop("jya(30) = {:.4}", jya(pai / 6))
```

...and LIPI translates it to Devanagari before compiling. All the keywords, builtins, and module names have phonetic Roman equivalents.

---

### Why this matters

India produced some of the most sophisticated mathematics in human history — zero, negative numbers, place-value notation, sine, Fibonacci sequences, combinatorics — centuries before these ideas reached Europe.

Modern programming languages are built entirely on Western mathematical vocabulary. LIPI is an attempt to ask: *what if the vocabulary came from the other tradition?*

It's not a replacement for Python or Rust. It's a statement of cultural ownership. It says: Indian students can learn to code in their own language, using their own mathematical heritage, and the names they see in the code are names they should be proud of.

---

**Open source. Written in Rust. 14 phases done.**

If you're interested in language design, Rust systems programming, or Indian history of mathematics — this might be your kind of project.

#Rust #ProgrammingLanguage #India #OpenSource #Devanagari #IndianMathematics #SystemsProgramming #Compiler #Hindi
