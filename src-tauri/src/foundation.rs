use serde::Serialize;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FoundationSummary {
    pub execution_authority: String,
    pub milestone: String,
}

impl FoundationSummary {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for FoundationSummary {
    fn default() -> Self {
        Self {
            execution_authority: "rust-local-core".to_owned(),
            milestone: "foundation".to_owned(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::FoundationSummary;

    #[test]
    fn creates_the_foundation_summary() {
        assert_eq!(
            FoundationSummary::new(),
            FoundationSummary {
                execution_authority: "rust-local-core".to_owned(),
                milestone: "foundation".to_owned(),
            },
        );
    }
}
