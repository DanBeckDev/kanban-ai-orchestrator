use rusqlite::{OptionalExtension, params};

use crate::domain::{ProjectAgentSettings, ProjectId};

use super::{EventStoreError, SqliteEventStore};

impl SqliteEventStore {
    pub fn save_project_agent_settings(
        &mut self,
        settings: ProjectAgentSettings,
    ) -> Result<ProjectAgentSettings, EventStoreError> {
        let settings_json = serde_json::to_string(&settings)?;
        self.connection.execute(
            "INSERT INTO project_agent_settings (project_id, settings_json)
             VALUES (?1, ?2)
             ON CONFLICT(project_id) DO UPDATE SET settings_json = excluded.settings_json",
            params![settings.project_id.0, settings_json],
        )?;
        Ok(settings)
    }

    pub fn project_agent_settings(
        &self,
        project_id: &ProjectId,
    ) -> Result<Option<ProjectAgentSettings>, EventStoreError> {
        self.connection
            .query_row(
                "SELECT settings_json FROM project_agent_settings WHERE project_id = ?1",
                [project_id.0.as_str()],
                |row| row.get::<_, String>(0),
            )
            .optional()?
            .map(|settings_json| Ok(serde_json::from_str(&settings_json)?))
            .transpose()
    }
}
