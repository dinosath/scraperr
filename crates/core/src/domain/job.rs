use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum JobStatus {
    Queued,
    Scraping,
    Completed,
    Failed,
}

impl std::fmt::Display for JobStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JobStatus::Queued => write!(f, "Queued"),
            JobStatus::Scraping => write!(f, "Scraping"),
            JobStatus::Completed => write!(f, "Completed"),
            JobStatus::Failed => write!(f, "Failed"),
        }
    }
}

impl std::str::FromStr for JobStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Queued" => Ok(JobStatus::Queued),
            "Scraping" => Ok(JobStatus::Scraping),
            "Completed" => Ok(JobStatus::Completed),
            "Failed" => Ok(JobStatus::Failed),
            _ => Err(format!("Unknown job status: {s}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Element {
    pub name: String,
    pub xpath: String,
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapturedElement {
    pub xpath: String,
    pub text: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobOptions {
    #[serde(default)]
    pub multi_page_scrape: bool,
    #[serde(default)]
    pub custom_headers: serde_json::Value,
    #[serde(default)]
    pub proxies: Vec<serde_json::Value>,
    pub site_map: Option<SiteMap>,
    #[serde(default)]
    pub collect_media: bool,
    #[serde(default)]
    pub custom_cookies: Vec<serde_json::Value>,
    #[serde(default)]
    pub return_html: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteMap {
    pub actions: Vec<Action>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Action {
    #[serde(rename = "type")]
    pub action_type: String,
    pub xpath: String,
    pub name: String,
    #[serde(default)]
    pub input: String,
    #[serde(default = "default_true")]
    pub do_once: bool,
}

fn default_true() -> bool {
    true
}

/// Fields that are allowed to be updated on a job.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdatableJobField {
    Status,
    Favorite,
    Chat,
    Result,
}

impl std::str::FromStr for UpdatableJobField {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "status" => Ok(UpdatableJobField::Status),
            "favorite" => Ok(UpdatableJobField::Favorite),
            "chat" => Ok(UpdatableJobField::Chat),
            "result" => Ok(UpdatableJobField::Result),
            _ => Err(format!("Field '{s}' is not updatable")),
        }
    }
}
