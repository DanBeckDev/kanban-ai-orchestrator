use rusqlite::{OptionalExtension, params};

use crate::agent::AgentProfile;

use super::{EventStoreError, SqliteEventStore};

impl SqliteEventStore {
    pub fn save_agent_profile(
        &mut self,
        profile: AgentProfile,
    ) -> Result<AgentProfile, EventStoreError> {
        profile
            .validate()
            .map_err(EventStoreError::InvalidAgentProfile)?;
        let profile_json = serde_json::to_string(&profile)?;
        self.connection.execute(
            "INSERT INTO agent_profiles (profile_name, profile_json)
             VALUES (?1, ?2)
             ON CONFLICT(profile_name) DO UPDATE SET profile_json = excluded.profile_json",
            params![profile.name, profile_json],
        )?;
        Ok(profile)
    }

    pub fn agent_profile(&self, name: &str) -> Result<Option<AgentProfile>, EventStoreError> {
        self.connection
            .query_row(
                "SELECT profile_json FROM agent_profiles WHERE profile_name = ?1",
                [name],
                |row| row.get::<_, String>(0),
            )
            .optional()?
            .map(|profile_json| Ok(serde_json::from_str(&profile_json)?))
            .transpose()
    }

    pub fn agent_profiles(&self) -> Result<Vec<AgentProfile>, EventStoreError> {
        let mut statement = self
            .connection
            .prepare("SELECT profile_json FROM agent_profiles ORDER BY profile_name")?;
        let profiles = statement.query_map([], |row| row.get::<_, String>(0))?;
        profiles
            .map(|profile_json| Ok(serde_json::from_str(&profile_json?)?))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        agent::{AgentProfile, AgentProfileKind},
        persistence::SqliteEventStore,
    };

    fn profile(name: &str, argument: &str) -> AgentProfile {
        AgentProfile {
            name: name.to_owned(),
            kind: AgentProfileKind::StructuredProcess,
            program: "agent-worker".to_owned(),
            arguments: vec![argument.to_owned()],
        }
    }

    #[test]
    fn stores_sorted_profiles_and_replaces_a_matching_name() {
        let mut store = SqliteEventStore::in_memory().expect("store should open");
        store
            .save_agent_profile(profile("zeta", "--old"))
            .expect("first profile should save");
        let replacement = profile("zeta", "--new");
        store
            .save_agent_profile(replacement.clone())
            .expect("replacement should save");
        store
            .save_agent_profile(profile("alpha", "--jsonl"))
            .expect("second profile should save");

        assert_eq!(
            store.agent_profile("zeta").expect("profile should load"),
            Some(replacement)
        );
        assert_eq!(
            store
                .agent_profiles()
                .expect("profiles should list")
                .into_iter()
                .map(|profile| profile.name)
                .collect::<Vec<_>>(),
            vec!["alpha", "zeta"]
        );
    }
}
