use crate::{
    desktop_execution_runtime::ExecutionRuntime,
    desktop_execution_runtime_support::{ExecutionRuntimeError, lock},
    domain::WorkItemId,
};

impl ExecutionRuntime {
    pub(crate) fn coordinate_after_execution(&self, work_item_id: &WorkItemId) {
        let board_id = lock(&self.service, "board service")
            .and_then(|service| {
                service
                    .work_item(work_item_id)
                    .map_err(ExecutionRuntimeError::Board)
            })
            .map(|work_item| work_item.work_item.board_id);
        if let Ok(board_id) = board_id {
            let _ = self.coordinate_board(&board_id.0);
        }
    }
}
