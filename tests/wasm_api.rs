// Verifies the WASM-facing API functions (used by LIPI Studio) return correct
// data when compiled natively (the wasm_bindgen attribute is cfg-gated off here).

#[test]
fn diagnostics_clean_and_error() {
    assert_eq!(lipi::lipi_diagnostics("बताओ 1\n"), "[]");
    let d = lipi::lipi_diagnostics("बताओ 1\nयदि\n");
    assert!(d.contains("\"line\":2"), "expected line 2 error, got {d}");
    assert!(d.starts_with("[{") && d.ends_with("}]"));
}

#[test]
fn symbols_extracted() {
    let src = "विधि जोड़ो(अ, ब):\n    फल अ + ब\nवर्ग बिंदु:\n    विधि बनाओ():\n        फल यह\n";
    let s = lipi::lipi_symbols(src);
    assert!(s.contains("\"name\":\"जोड़ो\""), "{s}");
    assert!(s.contains("\"kind\":\"function\""), "{s}");
    assert!(s.contains("\"name\":\"बिंदु\""), "{s}");
    assert!(s.contains("\"kind\":\"class\""), "{s}");
}

#[test]
fn completions_and_hover() {
    let c = lipi::lipi_completions();
    assert!(c.contains("\"label\":\"विधि\""), "{c}");
    assert!(c.contains("\"label\":\"उत्पन्न\""), "{c}");
    assert!(c.contains("\"label\":\"प्रतीक्षा\""), "{c}");
    assert!(lipi::lipi_hover("विधि").contains("function"));
    assert!(lipi::lipi_hover("मानचित्र").contains("builtin"));
    assert_eq!(lipi::lipi_hover("कुछ_अज्ञात"), "");
}

#[test]
fn run_source_works() {
    assert_eq!(lipi::run_source("बताओ 2 + 3\n"), "5");
    assert_eq!(lipi::run_source("बताओ सूची_में(गिनती(3))\nविधि गिनती(n):\n    i के लिए n में:\n        उत्पन्न i\n"), "[0, 1, 2]");
}
