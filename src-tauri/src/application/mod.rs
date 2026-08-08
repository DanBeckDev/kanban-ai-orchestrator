mod board_service;
mod board_snapshot;

pub use board_service::{
    AddDependencyRequest, BoardRepository, BoardService, BoardServiceError, CreateBoardRequest,
    CreateProjectRequest, CreateWorkItemRequest, TransitionWorkItemRequest,
};
pub use board_snapshot::{BoardActivity, BoardSnapshot, board_activity};

#[cfg(test)]
mod board_service_tests;
