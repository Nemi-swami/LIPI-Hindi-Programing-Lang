/// LIPI 2.0 — Karaka Type System
///
/// Sanskrit's 6 Karakas describe the ROLE of a noun in an action —
/// not merely its data type. This is the world's first implementation
/// of Paninian Karaka theory as a programming language type system.
///
/// Reference: Ashtadhyayi, Panini (~500 BCE), kāraka-prakarana

use std::collections::HashMap;

/// The six Karakas of Sanskrit grammar, each representing a semantic role.
#[derive(Debug, Clone, PartialEq)]
pub enum Karaka {
    /// कर्ता — the doer / agent who performs the action
    /// Example: "राम ने आम खाया" → राम is Karta
    Karta,

    /// कर्म — the object / patient that is acted upon
    /// Example: "राम ने आम खाया" → आम is Karma
    Karma,

    /// करण — the instrument / means by which the action is done
    /// Example: "राम ने चाकू से काटा" → चाकू is Karana
    Karana,

    /// सम्प्रदान — the recipient / beneficiary of the action
    /// Example: "राम ने श्याम को दिया" → श्याम is Sampradan
    Sampradan,

    /// अपादान — the source / point of departure
    /// Example: "पेड़ से फल गिरा" → पेड़ is Apadan
    Apadan,

    /// अधिकरण — the location / context where action happens
    /// Example: "वह घर में है" → घर is Adhikaran
    Adhikaran,

    /// No annotation — untyped (no warning issued for these)
    Unknown,
}

impl Karaka {
    /// Hindi name of the Karaka for error messages
    pub fn name(&self) -> &'static str {
        match self {
            Karaka::Karta     => "कर्ता",
            Karaka::Karma     => "कर्म",
            Karaka::Karana    => "करण",
            Karaka::Sampradan => "सम्प्रदान",
            Karaka::Apadan    => "अपादान",
            Karaka::Adhikaran => "अधिकरण",
            Karaka::Unknown   => "अज्ञात",
        }
    }

    pub fn is_known(&self) -> bool {
        !matches!(self, Karaka::Unknown)
    }
}

impl std::fmt::Display for Karaka {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Tracks Karaka annotations assigned to variables in the current scope.
/// Lives alongside the interpreter's value environment.
#[derive(Debug)]
pub struct KarakaEnv {
    map: HashMap<String, Karaka>,
}

impl KarakaEnv {
    pub fn new() -> Self {
        KarakaEnv { map: HashMap::new() }
    }

    /// Record the Karaka role of a named variable.
    pub fn annotate(&mut self, name: &str, karaka: Karaka) {
        if karaka.is_known() {
            self.map.insert(name.to_string(), karaka);
        }
    }

    /// Retrieve the Karaka of a variable (Unknown if not annotated).
    pub fn get(&self, name: &str) -> Karaka {
        self.map.get(name).cloned().unwrap_or(Karaka::Unknown)
    }

    /// Merge a child KarakaEnv into this one (used on scope exit).
    pub fn merge_child(&mut self, child: KarakaEnv) {
        for (k, v) in child.map {
            self.map.insert(k, v);
        }
    }

    /// Check that argument Karakas match parameter expectations.
    /// Emits soft warnings (to stderr) on mismatch — not hard errors.
    ///
    /// `params`        — list of (param_name, Option<expected_karaka>)
    /// `arg_var_names` — for each argument position, the variable name if
    ///                   the argument is an Ident, or None for literals
    pub fn check_call(
        &self,
        fn_name: &str,
        params: &[(String, Option<Karaka>)],
        arg_var_names: &[Option<String>],
    ) {
        for (i, (param_name, expected_opt)) in params.iter().enumerate() {
            let Some(expected) = expected_opt else { continue };
            let Some(Some(arg_name)) = arg_var_names.get(i) else { continue };

            let actual = self.get(arg_name);
            if actual.is_known() && &actual != expected {
                eprintln!(
                    "\x1b[33m[करक-चेतावनी]\x1b[0m विधि '{}': '{}' ({}) को {} अपेक्षित, '{}' ({}) मिला",
                    fn_name,
                    param_name, expected.name(),
                    expected.name(),
                    arg_name, actual.name(),
                );
            }
        }
    }
}
