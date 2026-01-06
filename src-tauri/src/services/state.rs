use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppState {
    pub current_position: u32,
    pub is_running: bool,
    pub is_paused: bool,
    pub last_session_id: String,
}

impl AppState {
    pub async fn load(db: &SqlitePool) -> Result<Self, sqlx::Error> {
        let rows: Vec<(String, String)> = sqlx::query_as(
            "SELECT key, value FROM app_state"
        )
        .fetch_all(db)
        .await?;

        let mut state = AppState::default();

        for (key, value) in rows {
            match key.as_str() {
                "current_position" => state.current_position = value.parse().unwrap_or(0),
                "is_running" => state.is_running = value == "true",
                "is_paused" => state.is_paused = value == "true",
                "last_session_id" => state.last_session_id = value,
                _ => {}
            }
        }

        Ok(state)
    }

    pub async fn save(&self, db: &SqlitePool) -> Result<(), sqlx::Error> {
        let pairs = [
            ("current_position", self.current_position.to_string()),
            ("is_running", self.is_running.to_string()),
            ("is_paused", self.is_paused.to_string()),
            ("last_session_id", self.last_session_id.clone()),
        ];

        for (key, value) in pairs {
            sqlx::query(
                "INSERT OR REPLACE INTO app_state (key, value) VALUES (?, ?)"
            )
            .bind(key)
            .bind(value)
            .execute(db)
            .await?;
        }

        Ok(())
    }

    #[allow(dead_code)]
    pub async fn clear(db: &SqlitePool) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM app_state")
            .execute(db)
            .await?;
        Ok(())
    }
}
