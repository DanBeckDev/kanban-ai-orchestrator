use rusqlite::{OptionalExtension, Transaction, params};

use crate::application::{BoardSnapshot, board_activity};
use crate::domain::{
    Board, BoardId, CreateWorkItemCommand, Dependency, DependencyGraph, DependencyId,
    MaterializedWorkItem, Project, ProjectId, RecordedWorkItemEvent,
};

use super::{BoardStoreError, SqliteEventStore};

const RECENT_ACTIVITY_LIMIT_PER_WORK_ITEM: u32 = 20;
const RECENT_EXECUTION_LIMIT_PER_WORK_ITEM: u32 = 20;
const RECENT_EVIDENCE_LIMIT_PER_WORK_ITEM: u32 = 20;

impl SqliteEventStore {
    pub fn create_project(&mut self, project: Project) -> Result<Project, BoardStoreError> {
        if self.project(&project.id)?.is_some() {
            return Err(BoardStoreError::ProjectAlreadyExists {
                project_id: project.id,
            });
        }

        let project_json = serde_json::to_string(&project)?;
        self.connection.execute(
            "INSERT INTO projects (project_id, project_json) VALUES (?1, ?2)",
            params![project.id.0, project_json],
        )?;
        Ok(project)
    }

    pub fn project(&self, project_id: &ProjectId) -> Result<Option<Project>, BoardStoreError> {
        let project_json = self
            .connection
            .query_row(
                "SELECT project_json FROM projects WHERE project_id = ?1",
                [project_id.0.as_str()],
                |row| row.get::<_, String>(0),
            )
            .optional()?;

        project_json
            .map(|serialized_project| serde_json::from_str(&serialized_project))
            .transpose()
            .map_err(BoardStoreError::from)
    }

    pub fn create_board(&mut self, board: Board) -> Result<Board, BoardStoreError> {
        self.require_project(&board.project_id)?;
        if self.board(&board.id)?.is_some() {
            return Err(BoardStoreError::BoardAlreadyExists { board_id: board.id });
        }

        let board_json = serde_json::to_string(&board)?;
        self.connection.execute(
            "INSERT INTO boards (board_id, project_id, board_json) VALUES (?1, ?2, ?3)",
            params![board.id.0, board.project_id.0, board_json],
        )?;
        Ok(board)
    }

    pub fn board(&self, board_id: &BoardId) -> Result<Option<Board>, BoardStoreError> {
        let board_json = self
            .connection
            .query_row(
                "SELECT board_json FROM boards WHERE board_id = ?1",
                [board_id.0.as_str()],
                |row| row.get::<_, String>(0),
            )
            .optional()?;

        board_json
            .map(|serialized_board| serde_json::from_str(&serialized_board))
            .transpose()
            .map_err(BoardStoreError::from)
    }

    pub fn boards_for_project(
        &self,
        project_id: &ProjectId,
    ) -> Result<Vec<Board>, BoardStoreError> {
        self.require_project(project_id)?;
        let mut statement = self
            .connection
            .prepare("SELECT board_json FROM boards WHERE project_id = ?1 ORDER BY board_id")?;
        let rows = statement.query_map([project_id.0.as_str()], |row| row.get::<_, String>(0))?;
        rows.map(|row| {
            let serialized_board = row?;
            Ok(serde_json::from_str(&serialized_board)?)
        })
        .collect()
    }

    pub fn create_board_work_item(
        &mut self,
        command: CreateWorkItemCommand,
    ) -> Result<RecordedWorkItemEvent, BoardStoreError> {
        self.require_board(&command.work_item.board_id)?;
        Ok(self.create_work_item(command)?)
    }

    pub fn add_board_dependency(
        &mut self,
        dependency: Dependency,
    ) -> Result<Dependency, BoardStoreError> {
        if let Some(recorded_dependency) = self.dependency(&dependency.id)? {
            return idempotent_dependency(recorded_dependency, dependency);
        }

        let board_id = self.require_dependency_board(&dependency)?;
        let work_items = self.work_items_for_board(&board_id)?;
        let mut graph = DependencyGraph::new(
            work_items
                .iter()
                .map(|materialized_work_item| materialized_work_item.work_item.id.clone()),
        );
        for existing_dependency in self.dependencies_for_board(&board_id)? {
            graph.add_dependency(existing_dependency)?;
        }
        graph.add_dependency(dependency.clone())?;

        let dependency_json = serde_json::to_string(&dependency)?;
        self.connection.execute(
            "INSERT INTO board_dependencies (
                dependency_id,
                board_id,
                upstream_work_item_id,
                downstream_work_item_id,
                dependency_json
             ) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                dependency.id.0,
                board_id.0,
                dependency.upstream_work_item_id.0,
                dependency.downstream_work_item_id.0,
                dependency_json,
            ],
        )?;
        Ok(dependency)
    }

    pub fn board_snapshot(&self, board_id: &BoardId) -> Result<BoardSnapshot, BoardStoreError> {
        let work_items = self.work_items_for_board(board_id)?;
        let work_item_ids = work_items
            .iter()
            .map(|materialized_work_item| materialized_work_item.work_item.id.clone())
            .collect::<Vec<_>>();
        Ok(BoardSnapshot {
            board: self.require_board(board_id)?,
            activity: self.activity_for(&work_items)?,
            work_items,
            dependencies: self.dependencies_for_board(board_id)?,
            executions: self.recent_executions_for_work_items(
                &work_item_ids,
                RECENT_EXECUTION_LIMIT_PER_WORK_ITEM,
            )?,
            evidence: self.recent_evidence_for_work_items(
                &work_item_ids,
                RECENT_EVIDENCE_LIMIT_PER_WORK_ITEM,
            )?,
        })
    }

    fn require_project(&self, project_id: &ProjectId) -> Result<Project, BoardStoreError> {
        self.project(project_id)?
            .ok_or_else(|| BoardStoreError::ProjectNotFound {
                project_id: project_id.clone(),
            })
    }

    fn require_board(&self, board_id: &BoardId) -> Result<Board, BoardStoreError> {
        self.board(board_id)?
            .ok_or_else(|| BoardStoreError::BoardNotFound {
                board_id: board_id.clone(),
            })
    }

    fn work_items_for_board(
        &self,
        board_id: &BoardId,
    ) -> Result<Vec<MaterializedWorkItem>, BoardStoreError> {
        Ok(self
            .all_materialized_work_items()?
            .into_iter()
            .filter(|materialized_work_item| materialized_work_item.work_item.board_id == *board_id)
            .collect())
    }

    fn activity_for(
        &self,
        work_items: &[MaterializedWorkItem],
    ) -> Result<Vec<crate::application::BoardActivity>, BoardStoreError> {
        let mut activity = Vec::new();
        for materialized_work_item in work_items {
            activity.extend(
                self.recent_work_item_events(
                    &materialized_work_item.work_item.id,
                    RECENT_ACTIVITY_LIMIT_PER_WORK_ITEM,
                )?
                .into_iter()
                .map(board_activity),
            );
        }
        activity.sort_by_key(|entry| entry.sequence);
        Ok(activity)
    }

    fn dependency(
        &self,
        dependency_id: &DependencyId,
    ) -> Result<Option<Dependency>, BoardStoreError> {
        let dependency_json = self
            .connection
            .query_row(
                "SELECT dependency_json FROM board_dependencies WHERE dependency_id = ?1",
                [dependency_id.0.as_str()],
                |row| row.get::<_, String>(0),
            )
            .optional()?;

        dependency_json
            .map(|serialized_dependency| serde_json::from_str(&serialized_dependency))
            .transpose()
            .map_err(BoardStoreError::from)
    }

    fn dependencies_for_board(
        &self,
        board_id: &BoardId,
    ) -> Result<Vec<Dependency>, BoardStoreError> {
        let mut statement = self.connection.prepare(
            "SELECT dependency_json
             FROM board_dependencies
             WHERE board_id = ?1
             ORDER BY dependency_id",
        )?;
        let rows = statement.query_map([board_id.0.as_str()], |row| row.get::<_, String>(0))?;
        rows.map(|row| {
            let serialized_dependency = row?;
            Ok(serde_json::from_str(&serialized_dependency)?)
        })
        .collect()
    }

    fn require_dependency_board(
        &self,
        dependency: &Dependency,
    ) -> Result<BoardId, BoardStoreError> {
        let upstream = self
            .materialized_work_item(&dependency.upstream_work_item_id)?
            .ok_or_else(|| BoardStoreError::WorkItemNotFound {
                work_item_id: dependency.upstream_work_item_id.clone(),
            })?;
        let downstream = self
            .materialized_work_item(&dependency.downstream_work_item_id)?
            .ok_or_else(|| BoardStoreError::WorkItemNotFound {
                work_item_id: dependency.downstream_work_item_id.clone(),
            })?;
        if upstream.work_item.board_id == downstream.work_item.board_id {
            Ok(upstream.work_item.board_id)
        } else {
            Err(BoardStoreError::CrossBoardDependency {
                dependency_id: dependency.id.clone(),
                upstream_board_id: upstream.work_item.board_id,
                downstream_board_id: downstream.work_item.board_id,
            })
        }
    }
}

pub(crate) fn create_protected_git_approval_schema(
    transaction: &Transaction<'_>,
) -> Result<(), rusqlite::Error> {
    transaction.execute_batch(
        "CREATE TABLE IF NOT EXISTS protected_git_approvals (
            approval_decision_id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL,
            work_item_id TEXT,
            git_action TEXT NOT NULL,
            approved_at TEXT NOT NULL,
            approval_json TEXT NOT NULL
        );
        CREATE INDEX IF NOT EXISTS protected_git_approvals_by_project
            ON protected_git_approvals (project_id, approved_at, approval_decision_id);",
    )
}

pub(crate) fn create_board_schema(transaction: &Transaction<'_>) -> Result<(), rusqlite::Error> {
    transaction.execute_batch(
        "CREATE TABLE IF NOT EXISTS projects (
            project_id TEXT PRIMARY KEY,
            project_json TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS boards (
            board_id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL,
            board_json TEXT NOT NULL
        );
        CREATE INDEX IF NOT EXISTS boards_by_project ON boards (project_id, board_id);
        CREATE TABLE IF NOT EXISTS board_dependencies (
            dependency_id TEXT PRIMARY KEY,
            board_id TEXT NOT NULL,
            upstream_work_item_id TEXT NOT NULL,
            downstream_work_item_id TEXT NOT NULL,
            dependency_json TEXT NOT NULL
        );
        CREATE INDEX IF NOT EXISTS board_dependencies_by_board
            ON board_dependencies (board_id, dependency_id);",
    )
}

fn idempotent_dependency(
    recorded_dependency: Dependency,
    dependency: Dependency,
) -> Result<Dependency, BoardStoreError> {
    if recorded_dependency == dependency {
        Ok(recorded_dependency)
    } else {
        Err(BoardStoreError::DependencyIdConflict {
            dependency_id: dependency.id,
        })
    }
}
