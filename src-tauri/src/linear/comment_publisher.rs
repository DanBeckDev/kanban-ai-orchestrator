use serde::{Deserialize, Serialize};

use super::LinearOAuthError;

const LINEAR_GRAPHQL_URL: &str = "https://api.linear.app/graphql";
const CREATE_COMMENT_MUTATION: &str = r#"
mutation CreateComment($input: CommentCreateInput!) {
  commentCreate(input: $input) {
    success
  }
}
"#;

pub trait LinearCommentPublisher {
    fn publish_comment(
        &self,
        access_token: &str,
        issue_id: &str,
        body: &str,
    ) -> Result<(), LinearOAuthError>;
}

pub struct ReqwestLinearCommentPublisher {
    client: reqwest::blocking::Client,
}

impl ReqwestLinearCommentPublisher {
    pub fn new() -> Self {
        Self {
            client: reqwest::blocking::Client::new(),
        }
    }
}

impl Default for ReqwestLinearCommentPublisher {
    fn default() -> Self {
        Self::new()
    }
}

impl LinearCommentPublisher for ReqwestLinearCommentPublisher {
    fn publish_comment(
        &self,
        access_token: &str,
        issue_id: &str,
        body: &str,
    ) -> Result<(), LinearOAuthError> {
        let response = self
            .client
            .post(LINEAR_GRAPHQL_URL)
            .bearer_auth(access_token)
            .json(&CommentRequest {
                query: CREATE_COMMENT_MUTATION,
                variables: CommentVariables {
                    input: CommentInput { issue_id, body },
                },
            })
            .send()
            .and_then(reqwest::blocking::Response::error_for_status)
            .and_then(reqwest::blocking::Response::text)
            .map_err(|error| LinearOAuthError::GraphQl(error.to_string()))?;
        comment_created_from_response(&response)
    }
}

#[derive(Serialize)]
struct CommentRequest<'value> {
    query: &'static str,
    variables: CommentVariables<'value>,
}

#[derive(Serialize)]
struct CommentVariables<'value> {
    input: CommentInput<'value>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CommentInput<'value> {
    issue_id: &'value str,
    body: &'value str,
}

#[derive(Deserialize)]
struct CommentResponse {
    data: Option<CommentData>,
    #[serde(default)]
    errors: Vec<GraphQlError>,
}

#[derive(Deserialize)]
struct CommentData {
    #[serde(rename = "commentCreate")]
    comment_create: CommentPayload,
}

#[derive(Deserialize)]
struct CommentPayload {
    success: bool,
}

#[derive(Deserialize)]
struct GraphQlError {
    message: String,
}

fn comment_created_from_response(response: &str) -> Result<(), LinearOAuthError> {
    let response = serde_json::from_str::<CommentResponse>(response).map_err(|_| {
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
    if response
        .data
        .is_some_and(|data| data.comment_create.success)
    {
        Ok(())
    } else {
        Err(LinearOAuthError::GraphQl(
            "Linear did not confirm comment delivery".to_owned(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::{CREATE_COMMENT_MUTATION, comment_created_from_response};

    #[test]
    fn accepts_only_an_explicit_linear_comment_success() {
        assert!(
            comment_created_from_response(r#"{"data":{"commentCreate":{"success":true}}}"#).is_ok()
        );
        for response in [
            r#"{"data":{"commentCreate":{"success":false}}}"#,
            r#"{"errors":[{"message":"Not authorized"}]}"#,
            r#"{"data":null}"#,
            "not JSON",
        ] {
            assert!(comment_created_from_response(response).is_err());
        }
    }

    #[test]
    fn keeps_the_mutation_boundary_small_and_typed() {
        assert!(CREATE_COMMENT_MUTATION.contains("commentCreate(input: $input)"));
        assert!(CREATE_COMMENT_MUTATION.contains("CommentCreateInput"));
        assert!(!CREATE_COMMENT_MUTATION.contains("issueUpdate"));
    }
}
