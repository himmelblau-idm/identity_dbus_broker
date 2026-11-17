use std::error::Error;
use std::fmt;

/// A category explicitly supplied by the broker implementation.
#[derive(Debug, Clone, Copy)]
pub enum BrokerErrorKind {
    Unexpected,
    InteractionRequired,
    ApiContractViolation,
    IncorrectConfiguration,
}

/// An error whose category can be conveyed to MSAL clients.
#[derive(Debug)]
pub struct BrokerError {
    pub kind: BrokerErrorKind,
    pub context: String,
}

impl BrokerError {
    pub fn new(kind: BrokerErrorKind, context: impl Into<String>) -> Self {
        Self {
            kind,
            context: context.into(),
        }
    }
}

impl fmt::Display for BrokerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.context)
    }
}

impl Error for BrokerError {}

pub(crate) fn error_response(
    error: &(dyn Error + 'static),
    token: bool,
) -> String {
    // These status and tag values are those used in the original broker fix.
    // Do not infer a category from error text: callers may have erased the cause.
    let kind = error
        .downcast_ref::<BrokerError>()
        .map(|error| error.kind)
        .unwrap_or(BrokerErrorKind::Unexpected);
    let (status, tag) = match kind {
        BrokerErrorKind::Unexpected => (0, 508_117_271u32),
        BrokerErrorKind::InteractionRequired => (2, 557_973_645),
        BrokerErrorKind::ApiContractViolation => (6, 508_117_271),
        BrokerErrorKind::IncorrectConfiguration => (9, 508_117_271),
    };
    let obj = serde_json::json!({
        "context": error.to_string(),
        "status": status,
        "subStatus": 0,
        "tag": tag,
    });
    if token {
        serde_json::json!({ "brokerTokenResponse": { "error": obj } })
            .to_string()
    } else {
        serde_json::json!({ "error": obj }).to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explicit_categories_are_serialized_in_both_envelopes() {
        for (kind, status, tag) in [
            (BrokerErrorKind::Unexpected, 0, 508_117_271),
            (BrokerErrorKind::InteractionRequired, 2, 557_973_645),
            (BrokerErrorKind::ApiContractViolation, 6, 508_117_271),
            (BrokerErrorKind::IncorrectConfiguration, 9, 508_117_271),
        ] {
            let error = BrokerError::new(kind, "example");
            for token in [true, false] {
                let value: serde_json::Value =
                    serde_json::from_str(&error_response(&error, token)).unwrap();
                let obj = if token {
                    &value["brokerTokenResponse"]["error"]
                } else {
                    &value["error"]
                };
                assert_eq!(obj, &serde_json::json!({
                    "context": "example", "status": status,
                    "subStatus": 0, "tag": tag
                }));
            }
        }
    }

    #[test]
    fn untyped_errors_are_not_misclassified() {
        let error = std::io::Error::new(
            std::io::ErrorKind::Other,
            "Failed to authenticate user: network unavailable",
        );
        let value: serde_json::Value =
            serde_json::from_str(&error_response(&error, true)).unwrap();
        assert_eq!(value["brokerTokenResponse"]["error"]["status"], 0);
        assert_eq!(value["brokerTokenResponse"]["error"]["tag"], 508_117_271);
    }
}
