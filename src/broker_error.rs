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
///
/// The category is retained through I/O wrappers and error source chains.
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
    // Do not infer a category from error text: callers may have erased the cause.
    let mut cause = error;
    let kind = loop {
        if let Some(error) = cause.downcast_ref::<BrokerError>() {
            break error.kind;
        }
        // io::Error::source() can skip the immediately wrapped error.
        if let Some(inner) = cause
            .downcast_ref::<std::io::Error>()
            .and_then(std::io::Error::get_ref)
        {
            cause = inner;
        } else if let Some(inner) = cause.source() {
            cause = inner;
        } else {
            break BrokerErrorKind::Unexpected;
        }
    };
    let (status, tag) = match kind {
        BrokerErrorKind::Unexpected => (0, 508_117_271u32),
        BrokerErrorKind::InteractionRequired => (2, 557_973_645),
        BrokerErrorKind::ApiContractViolation => (6, 508_117_271),
        BrokerErrorKind::IncorrectConfiguration => (9, 508_117_271),
    };
    let obj = serde_json::json!({
        "context": error.to_string(),
        // MSAL requires this integer even when no native error code is available.
        "errorCode": 0,
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

    #[derive(Debug)]
    struct ContextError(Box<dyn Error + Send + Sync>);

    impl fmt::Display for ContextError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "request failed: {}", self.0)
        }
    }

    impl Error for ContextError {
        fn source(&self) -> Option<&(dyn Error + 'static)> {
            Some(self.0.as_ref())
        }
    }

    #[test]
    fn explicit_categories_survive_wrappers_in_both_envelopes() {
        for (kind, status, tag) in [
            (BrokerErrorKind::Unexpected, 0, 508_117_271),
            (BrokerErrorKind::InteractionRequired, 2, 557_973_645),
            (BrokerErrorKind::ApiContractViolation, 6, 508_117_271),
            (BrokerErrorKind::IncorrectConfiguration, 9, 508_117_271),
        ] {
            let broker_error = || BrokerError::new(kind, "example");
            let errors: [Box<dyn Error + Send + Sync>; 4] = [
                Box::new(broker_error()),
                Box::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    broker_error(),
                )),
                Box::new(ContextError(Box::new(broker_error()))),
                Box::new(ContextError(Box::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    ContextError(Box::new(broker_error())),
                )))),
            ];
            for error in errors {
                for token in [true, false] {
                    let value: serde_json::Value = serde_json::from_str(
                        &error_response(error.as_ref(), token),
                    )
                    .unwrap();
                    let obj = if token {
                        &value["brokerTokenResponse"]["error"]
                    } else {
                        &value["error"]
                    };
                    assert_eq!(
                        obj,
                        &serde_json::json!({
                            "context": error.to_string(),
                            "errorCode": 0, "status": status,
                            "subStatus": 0, "tag": tag
                        })
                    );
                }
            }
        }
    }

    #[test]
    fn untyped_errors_are_not_misclassified() {
        let error = std::io::Error::new(
            std::io::ErrorKind::Other,
            "Failed to authenticate user: network unavailable",
        );
        for token in [true, false] {
            let value: serde_json::Value =
                serde_json::from_str(&error_response(&error, token)).unwrap();
            let obj = if token {
                &value["brokerTokenResponse"]["error"]
            } else {
                &value["error"]
            };
            assert_eq!(obj["context"], error.to_string());
            assert_eq!(obj["errorCode"].as_i64(), Some(0));
            assert_eq!(obj["status"].as_i64(), Some(0));
            assert_eq!(obj["tag"].as_i64(), Some(508_117_271));
        }
    }
}
