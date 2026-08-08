mod board_requests;
mod board_service;
mod board_snapshot;
mod execution_event_controller;

pub use board_requests::{
    AddDependencyRequest, CreateBoardRequest, CreateProjectRequest, CreateWorkItemRequest,
    RecordEvidenceRequest, RecordExecutionRequest, TransitionWorkItemRequest,
    UpdateExecutionRequest,
};
pub use board_service::{BoardRepository, BoardService, BoardServiceError};
pub use board_snapshot::{BoardActivity, BoardSnapshot, board_activity};
pub use execution_event_controller::{ExecutionEventController, ExecutionEventControllerError};

#[cfg(test)]
mod board_service_tests;

#[cfg(test)]
mod execution_event_controller_tests;
