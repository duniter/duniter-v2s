use sc_network::service::traits::ValidationResult;
use std::fmt::Display;

/// Clonable version of sc_network::service::traits::ValidationResult
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DuniterStreamValidationResult {
    /// Accept inbound substream.
    Accept,

    /// Reject inbound substream.
    Reject,
}

impl Display for DuniterStreamValidationResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DuniterStreamValidationResult::Accept => write!(f, "Accept"),
            DuniterStreamValidationResult::Reject => write!(f, "Reject"),
        }
    }
}

impl From<ValidationResult> for DuniterStreamValidationResult {
    fn from(result: ValidationResult) -> Self {
        match result {
            ValidationResult::Accept => DuniterStreamValidationResult::Accept,
            ValidationResult::Reject => DuniterStreamValidationResult::Reject,
        }
    }
}

impl From<DuniterStreamValidationResult> for ValidationResult {
    fn from(result: DuniterStreamValidationResult) -> Self {
        match result {
            DuniterStreamValidationResult::Accept => ValidationResult::Accept,
            DuniterStreamValidationResult::Reject => ValidationResult::Reject,
        }
    }
}
