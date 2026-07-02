//! Gradual type system — type hints for LIPI (Phase 18 #7).
//!
//! `TypeHint` is parse-only metadata attached to parameters, return types, and
//! annotated variables. The compiler and VM ignore it entirely; only the static
//! checker in `typecheck.rs` reads it. Annotations are optional — unannotated code
//! is treated as `Any` and never flagged (gradual typing).

/// A declared type. `Any` (कुछ_भी) is the gradual escape hatch — compatible with
/// everything. `Named` is a nominal class type, checked permissively.
#[derive(Debug, Clone, PartialEq)]
pub enum TypeHint {
    Number,
    Str,
    Bool,
    List,
    Dict,
    Nil,
    Any,
    Named(String),
}

impl TypeHint {
    /// Map a Devanagari type name (and its accepted aliases) to a `TypeHint`.
    /// Any unrecognised identifier becomes a nominal `Named` class type.
    pub fn from_name(name: &str) -> TypeHint {
        match name {
            "संख्या" | "अंक" => TypeHint::Number,
            "वाक्य" | "पाठ" => TypeHint::Str,
            "तर्क" | "बूल" => TypeHint::Bool,
            "सूची" => TypeHint::List,
            "कोश" => TypeHint::Dict,
            "शून्य" => TypeHint::Nil,
            "कुछ_भी" => TypeHint::Any,
            other => TypeHint::Named(other.to_string()),
        }
    }

    /// Human-facing Devanagari name (canonical) for diagnostics.
    pub fn name(&self) -> String {
        match self {
            TypeHint::Number => "संख्या".to_string(),
            TypeHint::Str => "वाक्य".to_string(),
            TypeHint::Bool => "तर्क".to_string(),
            TypeHint::List => "सूची".to_string(),
            TypeHint::Dict => "कोश".to_string(),
            TypeHint::Nil => "शून्य".to_string(),
            TypeHint::Any => "कुछ_भी".to_string(),
            TypeHint::Named(n) => n.clone(),
        }
    }

    /// Gradual compatibility: is a value of type `actual` acceptable where
    /// `self` (the expected type) is declared?
    ///
    /// - `Any` on either side → always compatible (the gradual escape hatch).
    /// - `Named` on either side → permissive (nominal types aren't statically
    ///   tracked precisely yet, so never flag them).
    /// - otherwise → the two concrete primitives must match exactly.
    ///
    /// A mismatch is therefore reported only when BOTH sides are known, concrete,
    /// disagreeing primitives.
    pub fn accepts(&self, actual: &TypeHint) -> bool {
        match (self, actual) {
            (TypeHint::Any, _) | (_, TypeHint::Any) => true,
            (TypeHint::Named(_), _) | (_, TypeHint::Named(_)) => true,
            (a, b) => a == b,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aliases_map_to_same_type() {
        assert_eq!(TypeHint::from_name("संख्या"), TypeHint::Number);
        assert_eq!(TypeHint::from_name("अंक"), TypeHint::Number);
        assert_eq!(TypeHint::from_name("वाक्य"), TypeHint::Str);
        assert_eq!(TypeHint::from_name("पाठ"), TypeHint::Str);
        assert_eq!(TypeHint::from_name("तर्क"), TypeHint::Bool);
        assert_eq!(TypeHint::from_name("बूल"), TypeHint::Bool);
        assert_eq!(TypeHint::from_name("कुछ_भी"), TypeHint::Any);
    }

    #[test]
    fn unknown_name_is_nominal() {
        assert_eq!(TypeHint::from_name("व्यक्ति"), TypeHint::Named("व्यक्ति".to_string()));
    }

    #[test]
    fn concrete_mismatch_rejected() {
        assert!(!TypeHint::Number.accepts(&TypeHint::Str));
        assert!(!TypeHint::Str.accepts(&TypeHint::Number));
        assert!(!TypeHint::Bool.accepts(&TypeHint::List));
    }

    #[test]
    fn concrete_match_accepted() {
        assert!(TypeHint::Number.accepts(&TypeHint::Number));
        assert!(TypeHint::List.accepts(&TypeHint::List));
    }

    #[test]
    fn any_is_compatible_both_ways() {
        assert!(TypeHint::Any.accepts(&TypeHint::Number));
        assert!(TypeHint::Number.accepts(&TypeHint::Any));
    }

    #[test]
    fn named_is_permissive() {
        assert!(TypeHint::Named("क".into()).accepts(&TypeHint::Number));
        assert!(TypeHint::Number.accepts(&TypeHint::Named("क".into())));
    }
}
