use std::path::Path;

use serde::Serialize;

use crate::domain::{Board, Project};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BoardLibraryRecord {
    pub board: Board,
    pub project: Project,
    pub last_opened_at: Option<String>,
    pub attention: BoardAttentionSummary,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BoardAttentionSummary {
    pub active_work_item_count: u32,
    pub needs_attention_count: u32,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BoardLibraryEntry {
    pub board_id: String,
    pub name: String,
    pub repository_name: String,
    pub repository_available: bool,
    pub last_opened_at: Option<String>,
    pub attention: BoardAttentionSummary,
}

impl BoardLibraryEntry {
    pub fn from_record(record: BoardLibraryRecord) -> Self {
        Self {
            board_id: record.board.id.0,
            name: record.board.name,
            repository_name: repository_name(&record.project.repository_path),
            repository_available: repository_available(&record.project.repository_path),
            last_opened_at: record.last_opened_at,
            attention: record.attention,
        }
    }
}

pub fn sort_board_library(entries: &mut [BoardLibraryEntry]) {
    entries.sort_by(|left, right| {
        right
            .last_opened_at
            .cmp(&left.last_opened_at)
            .then_with(|| left.name.cmp(&right.name))
            .then_with(|| left.board_id.cmp(&right.board_id))
    });
}

pub fn repository_available(repository_path: &str) -> bool {
    Path::new(repository_path).is_dir()
}

fn repository_name(repository_path: &str) -> String {
    Path::new(repository_path)
        .file_name()
        .filter(|name| !name.is_empty())
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| repository_path.to_owned())
}

#[cfg(test)]
mod tests {
    use crate::domain::{Board, BoardId, Project, ProjectId, SchemaMetadata};

    use super::{BoardAttentionSummary, BoardLibraryEntry, BoardLibraryRecord, sort_board_library};

    fn record(id: &str, name: &str, last_opened_at: Option<&str>) -> BoardLibraryRecord {
        BoardLibraryRecord {
            board: Board {
                schema: SchemaMetadata::current(),
                id: BoardId::from(id),
                project_id: ProjectId::from("project-1"),
                name: name.to_owned(),
            },
            project: Project {
                schema: SchemaMetadata::current(),
                id: ProjectId::from("project-1"),
                name: "Project".to_owned(),
                repository_path: "/projects/example".to_owned(),
                base_ref: "main".to_owned(),
                policy_set_id: "standard".to_owned(),
            },
            last_opened_at: last_opened_at.map(str::to_owned),
            attention: BoardAttentionSummary::default(),
        }
    }

    #[test]
    fn sorts_recent_boards_before_never_opened_boards_with_stable_name_ties() {
        let mut entries = vec![
            BoardLibraryEntry::from_record(record("board-3", "Zulu", None)),
            BoardLibraryEntry::from_record(record("board-2", "Alpha", None)),
            BoardLibraryEntry::from_record(record(
                "board-1",
                "Middle",
                Some("2026-08-09T08:00:00Z"),
            )),
        ];

        sort_board_library(&mut entries);

        assert_eq!(
            entries
                .iter()
                .map(|entry| entry.name.as_str())
                .collect::<Vec<_>>(),
            ["Middle", "Alpha", "Zulu"]
        );
    }
}
