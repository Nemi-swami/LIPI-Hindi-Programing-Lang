# LIPI Feature Extraction — From Hindu Scriptures Research
**Source:** hindu-scriptures-research.md  
**Purpose:** Concrete, actionable features to add to LIPI language

---

## IMMEDIATE ADDITIONS (Low effort, high authenticity)

### 1. New Built-in Constants
```lipi
बताओ पाई          # already done (π)
बताओ अनंत         # already done (∞)
बताओ शून्य         # 0 (as named constant, from Brahmagupta)
बताओ सत्य          # already: true
बताओ असत्य         # already: false
```

### 2. New Number Suffixes (beyond लाख/करोड़)
From Vedic numerical system:
```lipi
क है 1 करोड़        # 10^7 — already done
क है 1 अरब         # 10^9
क है 1 खरब         # 10^10
क है 1 नील         # 10^11
क है 1 पद्म         # 10^14
```

### 3. `भारत.छन्दस्` Module — Pingala's Binary
From Chandahshastra (~200 BCE) — Pingala invented binary notation for meter:
```lipi
आयात भारत.छन्दस्

# Binary encoding: L (laghu/light=0), G (guru/heavy=1)
बताओ मात्रा_मान("LGLG")   # 0101 binary = 5
बताओ छन्द_संख्या(4, 5)    # 5th meter of 4 syllables (Pingala's uddhishta)
बताओ मेरु_प्रस्तार(5)     # Row 5 of Pascal's triangle (Meru Prastara)
बताओ विरहांक(10)           # Fibonacci(10) = 55 — already in भारत.गणित!
```

### 4. `भारत.न्याय` Module — Formal Logic (Nyaya Sutras)
From 16 categories of Nyaya school:
```lipi
आयात भारत.न्याय

# 5-part syllogism (Pancavayava Anumana)
# प्रतिज्ञा: the claim
# हेतु: the reason  
# उदाहरण: the example
# उपनय: the application
# निगमन: the conclusion

# Type: Hetvabhasa (fallacy) detection
# Vyapti: universal rule (pervasion)
```

### 5. `भारत.शुल्ब` Module — Sulbasutras Geometry
From Baudhayana Sulbasutra (~800 BCE):
```lipi
आयात भारत.शुल्ब

बताओ वर्गमूल_शुल्ब(2)      # √2 = 1.4142156... (Baudhayana's approximation)
बताओ कर्ण(3, 4)             # Pythagorean hypotenuse = 5
बताओ वर्ग_क्षेत्र(5)        # Area of square with side 5
बताओ वृत्त_वर्गीकरण(5)     # Square approximating circle of radius 5
```

### 6. `भारत.नाट्य` Module — Rasa/Sentiment
From Bharata Muni's Natyashastra (9 Rasas):
```lipi
आयात भारत.नाट्य

# 9 Rasas as named constants
बताओ रस.शृंगार    # 1: love/beauty
बताओ रस.हास्य     # 2: humor
बताओ रस.करुण      # 3: sorrow
बताओ रस.रौद्र     # 4: fury
बताओ रस.वीर       # 5: heroism
बताओ रस.भयानक     # 6: fear/terror
बताओ रस.बीभत्स    # 7: disgust
बताओ रस.अद्भुत    # 8: wonder
बताओ रस.शान्त     # 9: peace
```

---

## MEDIUM ADDITIONS (New keywords/syntax)

### 7. `स्यात्` — Optional/Maybe Type (Jain Syadvada)
From Jain philosophy's conditional predication:
```lipi
विधि खोजो(सूची, लक्ष्य):
    i के लिए सूची में:
        यदि i बराबर लक्ष्य:
            फल स्यात् i        # "perhaps this value exists"
    फल स्यात्               # "perhaps nothing" = None/null

परिणाम है खोजो([1,2,3], 5)
यदि परिणाम स्यात् नहीं:
    बताओ "नहीं मिला"
```

### 8. `शुद्ध` — Pure Function Declaration
From Advaita Vedanta's "shuddha" (pure, untainted):
```lipi
शुद्ध विधि वर्गफल(अ):         # pure function — no side effects
    फल अ * अ

# Compiler can safely memoize शुद्ध functions
```

### 9. `साक्षी` — Observer/Witness Mode (Samkhya Purusha)
From Samkhya philosophy's Purusha as pure witness:
```lipi
साक्षी:                        # read-only scope — no mutation allowed
    बताओ क + 1              # OK: compute and print
    क है क + 1              # ERROR: cannot mutate in साक्षी block
```

### 10. `कुट्टक` Built-in — Aryabhata's Algorithm
Aryabhata's "pulverizer" (linear Diophantine solver, ~499 CE):
```lipi
# Solve ax + by = c for integers x, y
परिणाम है कुट्टक(17, 5, 1)   # extended GCD(17,5); returns [x, y, gcd]
बताओ परिणाम                    # [3, -10, 1] since 17×3 + 5×(-10) = 1
```

### 11. `चक्रवाल` Built-in — Bhaskara's Pell Solver
Chakravala algorithm for x² - Dy² = 1 (~1150 CE):
```lipi
आयात भारत.गणित
बताओ चक्रवाल(2)    # Solve x² - 2y² = 1 → [3, 2] (3² - 2×2² = 1)
बताओ चक्रवाल(61)   # Famous case: [1766319049, 226153980]
```

---

## PHILOSOPHICAL/AESTHETIC FEATURES

### 12. Three Guna Execution Modes
From Samkhya/Gita's Trigunas (three qualities of nature):
```lipi
# Execution quality markers on functions
तामस विधि धीमा_काम():    # Tamas: lazy/deferred execution
    # ... heavy computation only when needed

राजस विधि सक्रिय_काम():  # Rajas: eager/active execution
    # ... immediate computation

सात्त्विक विधि शुद्ध_काम(): # Sattva: pure/optimal execution  
    # ... compiler can fully optimize
```

### 13. Panchakosha (5-Layer Architecture)
From Taittiriya Upanishad's 5 sheaths:
```lipi
# Docstring-style layer annotation for modules
।। अन्नमय: data_layer ।।      # Layer 1: physical/data
।। प्राणमय: runtime_layer ।।   # Layer 2: energy/execution
।। मनोमय: logic_layer ।।      # Layer 3: mind/logic
।। विज्ञानमय: type_layer ।।   # Layer 4: discrimination/types
।। आनन्दमय: user_layer ।।     # Layer 5: bliss/UX
```

---

## STDLIB ENRICHMENTS

### 14. Enhanced `भारत.गणित` — Add Missing Classics

Already have: ज्या, कोज्या, विरहांक, संयोजन, ब्रह्मगुप्त_क्षेत्र, हेरॉन_क्षेत्र, etc.

**Add these from research:**
```lipi
# From Aryabhatiya (499 CE)
बताओ कुट्टक(a, b)          # Extended GCD / linear Diophantine
बताओ आर्यभट_योग(n)         # Sum 1+2+...+n = n(n+1)/2
बताओ वर्ग_योग(n)           # Sum 1²+2²+...+n² = n(n+1)(2n+1)/6
बताओ घन_योग(n)             # Sum 1³+2³+...+n³ = [n(n+1)/2]²

# From Brahmasphutasiddhanta (628 CE)  
बताओ चक्रवाल(D)            # Pell equation solver
बताओ ब्रह्मगुप्त_गुणन(a, b, n)  # Brahmagupta-Fibonacci identity

# From Lilavati (1150 CE)
बताओ ईएमआई_भास्कर(p, r, n)  # EMI = p*r*(1+r)^n/((1+r)^n-1) — already done!
बताओ क्षेत्रफल_त्रिभुज(a, b, c)  # Heron's formula — already done!

# From Pingala's Chandahshastra
बताओ मेरु_पंक्ति(n)         # nth row of Pascal's/Meru triangle
बताओ द्विआधार(n)            # Pingala's binary encoding of nth meter
```

### 15. `भारत.ज्योतिष` Module — Astronomy/Calendar
From Vedanga Jyotisha and Surya Siddhanta:
```lipi
आयात भारत.ज्योतिष

बताओ नक्षत्र_नाम(1)        # 1st of 27 nakshatras: "अश्विनी"
बताओ तिथि_आज()             # Today's lunar date (tithi)
बताओ युग_वर्ष()             # Which of the 4 Yugas we're in
बताओ सूर्योदय("दिल्ली")    # Sunrise time for location
```

**27 Nakshatras (for enum):**
अश्विनी, भरणी, कृत्तिका, रोहिणी, मृगशिरा, आर्द्रा, पुनर्वसु, पुष्य, आश्लेषा,
मघा, पूर्वाफाल्गुनी, उत्तराफाल्गुनी, हस्त, चित्रा, स्वाति, विशाखा, अनुराधा,
ज्येष्ठा, मूल, पूर्वाषाढा, उत्तराषाढा, श्रवण, धनिष्ठा, शतभिषा, पूर्वाभाद्रपद,
उत्तराभाद्रपद, रेवती

---

## ROMAN INPUT MAPPINGS TO ADD

For `src/roman.rs`, new keyword mappings from this research:

| Roman | Devanagari | Meaning |
|-------|-----------|---------|
| `shuddha` | `शुद्ध` | pure (function) |
| `syat` | `स्यात्` | maybe/optional |
| `sakshi` | `साक्षी` | observer/witness |
| `sattvic` | `सात्त्विक` | pure/optimal mode |
| `rajasic` | `राजस` | active mode |
| `tamasic` | `तामस` | lazy mode |
| `kuttak` | `कुट्टक` | extended GCD |
| `chakraval` | `चक्रवाल` | Pell solver |
| `meru` | `मेरु` | Pascal's triangle |
| `nakshatra` | `नक्षत्र` | lunar mansion |
| `rasa` | `रस` | aesthetic quality |
| `bhav` | `भाव` | internal state |
| `dharma` | `धर्म` | type/law |
| `karma` | `कर्म` | action/operation |
| `moksha` | `मोक्ष` | liberation/free |
| `vidya` | `विद्या` | knowledge/library |
| `shastra` | `शास्त्र` | science/module |
| `yukti` | `युक्ति` | algorithm/strategy |
| `upaya` | `उपाय` | approach/method |
| `viveka` | `विवेक` | discrimination |
| `siddhi` | `सिद्धि` | capability/feature |
| `shakti` | `शक्ति` | power/energy |

---

## PRIORITY ORDER FOR IMPLEMENTATION

1. **`कुट्टक`** — Aryabhata's extended GCD (pure algorithm, no new syntax)
2. **`मेरु_पंक्ति`** — Pascal's triangle row (pure math)
3. **`चक्रवाल`** — Pell equation (pure math, historically significant)
4. **27 नक्षत्र** enum — astronomical enum constant  
5. **`भारत.नाट्य`** — 9 Rasa constants (simple enum)
6. **`स्यात्`** — optional type (requires type system work)
7. **`शुद्ध`** — pure function annotation (compiler optimization hint)
8. **`भारत.छन्दस्`** — Pingala binary (interesting for binary ops)
9. **Three Guna modes** — execution quality (advanced feature)
10. **`साक्षी`** — read-only scope (requires compiler enforcement)

---

*Extracted from full research in hindu-scriptures-research.md*
