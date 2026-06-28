use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct UserSession {
    #[serde(rename = "UserId")]
       pub user_id: i64,
    #[serde(rename = "ProjectId")]
       pub project_id: i64,
       #[serde(rename="Config")]
    pub config_id: i64,
}


impl UserSession {
    pub fn new(user_id: i64, project_id: i64, config_id: i64) -> Self {
        Self {
            user_id,
            project_id,
            config_id,
        }
    }
}


#[derive(Debug, Serialize, Deserialize)]
pub struct SaveProjectConfigPathRarams {
    pub fetch : bool,
}