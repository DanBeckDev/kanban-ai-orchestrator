mod agent_profile_service;
mod board_requests;
mod board_service;
mod board_snapshot;
mod completion_evidence_service;
mod execution_event_controller;
mod execution_launch;
mod execution_policy_service;
mod execution_service;
mod execution_start_request;
mod review_check_request;
mod review_service;

pub use agent_profile_service::AgentProfileServiceError;
pub use board_requests::{
    AddDependencyRequest, CreateBoardRequest, CreateProjectRequest, CreateWorkItemRequest,
    RecordEvidenceRequest, RecordExecutionRequest, TransitionWorkItemRequest,
    UpdateExecutionRequest,
};
pub use board_service::{BoardRepository, BoardService, BoardServiceError};
pub use board_snapshot::{BoardActivity, BoardSnapshot, board_activity};
pub use execution_event_controller::{ExecutionEventController, ExecutionEventControllerError};
pub use execution_launch::{
    ExecutionLaunchError, ExecutionLaunchPreparation, prepare_execution_launch,
};
pub use execution_start_request::StartExecutionRequest;
pub use review_check_request::RecordReviewCheckRequest;

#[cfg(test)]
mod board_service_tests;

#[cfg(test)]
mod execution_event_controller_tests;

#[cfg(test)]
mod execution_launch_tests;
