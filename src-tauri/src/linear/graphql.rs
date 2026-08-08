use serde::{Deserialize, Serialize};

use super::LinearOAuthError;

const LINEAR_GRAPHQL_URL: &str = "https://api.linear.app/graphql";
const ASSIGNED_ISSUES_QUERY: &str = r#"
query AssignedIssues {
  viewer {
    assignedIssues(first: 50, orderBy: updatedAt) {
      nodes {
        id
        identifier
        title
        url
      }
    }
  }
}
"#;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LinearIssueSummary {
    pub id: String,
    pub identifier: String,
    pub title: String,
    pub url: String,
}

pub trait LinearGraphQlTransport {
    fn execute(&self, access_token: &str, query: &str) -> Result<String, LinearOAuthError>;
}

pub struct ReqwestLinearGraphQlTransport {
    client: reqwest::blocking::Client,
}

impl ReqwestLinearGraphQlTransport {
    pub fn new() -> Self {
        Self {
            client: reqwest::blocking::Client::new(),
        }
    }
}

impl Default for ReqwestLinearGraphQlTransport {
    fn default() -> Self {
        Self::new()
    }
}

impl LinearGraphQlTransport for ReqwestLinearGraphQlTransport {
    fn execute(&self, access_token: &str, query: &str) -> Result<String, LinearOAuthError> {
        self.client
            .post(LINEAR_GRAPHQL_URL)
            .bearer_auth(access_token)
            .json(&GraphQlRequest { query })
            .send()
            .and_then(reqwest::blocking::Response::error_for_status)
            .and_then(reqwest::blocking::Response::text)
            .map_err(|error| LinearOAuthError::GraphQl(error.to_string()))
    }
}

pub struct LinearIssueReader<Transport> {
    transport: Transport,
}

impl<Transport> LinearIssueReader<Transport>
where
    Transport: LinearGraphQlTransport,
{
    pub fn new(transport: Transport) -> Self {
        Self { transport }
    }

    pub fn assigned_issues(
        &self,
        access_token: &str,
    ) -> Result<Vec<LinearIssueSummary>, LinearOAuthError> {
        let response = self
            .transport
            .execute(access_token, ASSIGNED_ISSUES_QUERY)?;
        assigned_issues_from_response(&response)
    }
}

#[derive(Serialize)]
struct GraphQlRequest<'query> {
    query: &'query str,
}

#[derive(Deserialize)]
struct GraphQlResponse {
    data: Option<AssignedIssuesData>,
    #[serde(default)]
    errors: Vec<GraphQlError>,
}

#[derive(Deserialize)]
struct GraphQlError {
    message: String,
}

#[derive(Deserialize)]
struct AssignedIssuesData {
    viewer: AssignedIssuesViewer,
}

#[derive(Deserialize)]
struct AssignedIssuesViewer {
    #[serde(rename = "assignedIssues")]
    assigned_issues: IssueConnection,
}

#[derive(Deserialize)]
struct IssueConnection {
    nodes: Vec<IssueNode>,
}

#[derive(Deserialize)]
struct IssueNode {
    id: String,
    identifier: String,
    title: String,
    url: String,
}

fn assigned_issues_from_response(
    response: &str,
) -> Result<Vec<LinearIssueSummary>, LinearOAuthError> {
    let response = serde_json::from_str::<GraphQlResponse>(response).map_err(|_| {
        LinearOAuthError::GraphQl("Linear returned an invalid JSON response".to_owned())
    })?;
    if !response.errors.is_empty() {
        return Err(LinearOAuthError::GraphQl(
            response
                .errors
                .into_iter()
                .map(|error| error.message)
                .collect::<Vec<_>>()
                .join("; "),
        ));
    }
    let data = response
        .data
        .ok_or_else(|| LinearOAuthError::GraphQl("Linear response had no data".to_owned()))?;
    data.viewer
        .assigned_issues
        .nodes
        .into_iter()
        .map(LinearIssueSummary::try_from)
        .collect()
}

impl TryFrom<IssueNode> for LinearIssueSummary {
    type Error = LinearOAuthError;

    fn try_from(issue: IssueNode) -> Result<Self, Self::Error> {
        if [
            issue.id.as_str(),
            issue.identifier.as_str(),
            issue.title.as_str(),
            issue.url.as_str(),
        ]
        .iter()
        .any(|value| value.trim().is_empty())
        {
            return Err(LinearOAuthError::GraphQl(
                "Linear returned an incomplete issue summary".to_owned(),
            ));
        }
        Ok(Self {
            id: issue.id,
            identifier: issue.identifier,
            title: issue.title,
            url: issue.url,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use super::{
        ASSIGNED_ISSUES_QUERY, LinearGraphQlTransport, LinearIssueReader, LinearOAuthError,
        assigned_issues_from_response,
    };

    #[derive(Default)]
    struct FakeTransport {
        requested_queries: std::cell::RefCell<Vec<String>>,
        response: RefCell<String>,
    }

    impl FakeTransport {
        fn with_response(response: &str) -> Self {
            Self {
                requested_queries: RefCell::default(),
                response: RefCell::new(response.to_owned()),
            }
        }
    }

    impl LinearGraphQlTransport for FakeTransport {
        fn execute(&self, _access_token: &str, query: &str) -> Result<String, LinearOAuthError> {
            self.requested_queries.borrow_mut().push(query.to_owned());
            Ok(self.response.borrow().clone())
        }
    }

    struct FailingTransport;

    impl LinearGraphQlTransport for FailingTransport {
        fn execute(&self, _access_token: &str, _query: &str) -> Result<String, LinearOAuthError> {
            Err(LinearOAuthError::GraphQl("not authorized".to_owned()))
        }
    }

    #[test]
    fn reads_a_bounded_read_only_page_of_assigned_issue_summaries() {
        let transport = FakeTransport::with_response(
            r#"{"data":{"viewer":{"assignedIssues":{"nodes":[{"id":"d290f1ee-6c54-4b01-90e6-d701748f0851","identifier":"KAN-42","title":"Connect Linear","url":"https://linear.app/acme/issue/KAN-42/connect-linear"}]}}}}"#,
        );
        let reader = LinearIssueReader::new(transport);

        let issues = reader
            .assigned_issues("access-token")
            .expect("Linear response should parse");

        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].identifier, "KAN-42");
        let query = &reader.transport.requested_queries.borrow()[0];
        assert!(query.contains("assignedIssues(first: 50, orderBy: updatedAt)"));
        for field in ["id", "identifier", "title", "url"] {
            assert!(query.contains(field));
        }
        assert!(!query.contains("mutation"));
    }

    #[test]
    fn rejects_graphql_errors_and_responses_without_usable_data() {
        for response in [
            r#"{"errors":[{"message":"Not authorized"}]}"#,
            r#"{"data":null}"#,
            "not JSON",
        ] {
            assert!(assigned_issues_from_response(response).is_err());
        }
    }

    #[test]
    fn rejects_incomplete_issue_summaries() {
        let result = assigned_issues_from_response(
            r#"{"data":{"viewer":{"assignedIssues":{"nodes":[{"id":"d290f1ee-6c54-4b01-90e6-d701748f0851","identifier":"KAN-42","title":"","url":"https://linear.app/acme/issue/KAN-42/connect-linear"}]}}}}"#,
        );

        assert!(matches!(result, Err(LinearOAuthError::GraphQl(_))));
    }

    #[test]
    fn preserves_transport_failures_for_the_connection_panel() {
        let reader = LinearIssueReader::new(FailingTransport);

        assert_eq!(
            reader.assigned_issues("access-token"),
            Err(LinearOAuthError::GraphQl("not authorized".to_owned()))
        );
    }

    #[test]
    fn keeps_the_operation_text_as_a_read_only_contract() {
        assert!(ASSIGNED_ISSUES_QUERY.starts_with("\nquery AssignedIssues"));
        assert!(!ASSIGNED_ISSUES_QUERY.contains("input"));
    }
}
