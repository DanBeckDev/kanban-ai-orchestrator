use crate::{
    application::RecordEvidenceRequest,
    desktop_execution_runtime::ExecutionRuntime,
    desktop_execution_runtime_support::{lock, timestamp},
    domain::{EvidenceKind, EvidenceResult, Execution},
    workspace::{ReviewArtifacts, WorkspaceManager},
};

impl ExecutionRuntime {
    pub(crate) fn record_review_artifacts(&self, execution: &Execution) {
        let requests = match self.collect_review_artifacts(execution) {
            Ok(artifacts) => evidence_requests(execution, artifacts),
            Err(reason) => vec![collection_failure_request(execution, reason)],
        };
        for request in requests {
            let Ok(mut service) = lock(&self.service, "board service") else {
                return;
            };
            let _ = service.record_evidence(request);
        }
    }

    fn collect_review_artifacts(&self, execution: &Execution) -> Result<ReviewArtifacts, String> {
        let project = lock(&self.service, "board service")
            .map_err(|error| error.to_string())?
            .project_for_work_item(&execution.work_item_id)
            .map_err(|error| error.to_string())?;
        WorkspaceManager::new(&project, &self.workspace_root)
            .and_then(|manager| manager.collect_review_artifacts(execution))
            .map_err(|error| error.to_string())
    }
}

fn evidence_requests(
    execution: &Execution,
    artifacts: ReviewArtifacts,
) -> Vec<RecordEvidenceRequest> {
    let mut requests = Vec::new();
    if let Some(commit) = artifacts.head_commit {
        requests.push(RecordEvidenceRequest {
            evidence_id: format!("{}-git-commit", execution.id.0),
            work_item_id: execution.work_item_id.0.clone(),
            kind: EvidenceKind::Commit,
            result: EvidenceResult::Recorded,
            summary: format!("Task worktree HEAD commit: {commit}"),
            recorded_at: timestamp(),
        });
    }
    let diff_sections = [
        artifacts
            .committed_diff_stat
            .map(|stat| format!("Committed changes since the project base:\n{stat}")),
        artifacts
            .working_diff_stat
            .map(|stat| format!("Uncommitted changes:\n{stat}")),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>();
    if !diff_sections.is_empty() {
        requests.push(RecordEvidenceRequest {
            evidence_id: format!("{}-git-diff", execution.id.0),
            work_item_id: execution.work_item_id.0.clone(),
            kind: EvidenceKind::Diff,
            result: EvidenceResult::Recorded,
            summary: diff_sections.join("\n\n"),
            recorded_at: timestamp(),
        });
    }
    requests
}

fn collection_failure_request(execution: &Execution, reason: String) -> RecordEvidenceRequest {
    RecordEvidenceRequest {
        evidence_id: format!("{}-git-inspection", execution.id.0),
        work_item_id: execution.work_item_id.0.clone(),
        kind: EvidenceKind::Check,
        result: EvidenceResult::Failed,
        summary: format!("Git review evidence could not be collected: {reason}"),
        recorded_at: timestamp(),
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        domain::{
            EvidenceKind, ExecutionId, ExecutionStatus, ExecutionUsage, SchemaMetadata, WorkItemId,
        },
        workspace::ReviewArtifacts,
    };

    use super::evidence_requests;

    #[test]
    fn maps_review_artifacts_to_typed_durable_evidence() {
        let execution = crate::domain::Execution {
            schema: SchemaMetadata::current(),
            id: ExecutionId::from("execution-1"),
            work_item_id: WorkItemId::from("task-1"),
            adapter_name: "structured-worker".to_owned(),
            status: ExecutionStatus::AwaitingReview,
            session_id: Some("session-1".to_owned()),
            workspace_path: "/workspaces/task-1".to_owned(),
            usage: ExecutionUsage {
                input_tokens: 0,
                output_tokens: 0,
                cost_micros: None,
            },
            last_event_sequence: 2,
        };

        let requests = evidence_requests(
            &execution,
            ReviewArtifacts {
                head_commit: Some("abc123".to_owned()),
                committed_diff_stat: Some(" README.md | 1 +".to_owned()),
                working_diff_stat: Some(" README.md | 1 +".to_owned()),
            },
        );

        assert_eq!(requests.len(), 2);
        assert_eq!(requests[0].evidence_id, "execution-1-git-commit");
        assert_eq!(requests[0].kind, EvidenceKind::Commit);
        assert_eq!(requests[1].evidence_id, "execution-1-git-diff");
        assert_eq!(requests[1].kind, EvidenceKind::Diff);
        assert!(requests[1].summary.contains("Committed changes"));
        assert!(requests[1].summary.contains("Uncommitted changes"));
    }
}
