mod read;
mod write;

pub type CPXVersion = [u8; 4];

#[derive(Debug, Clone)]
pub struct CampaignHeader {
    pub(crate) version: CPXVersion,
    pub(crate) name: String,
    pub(crate) num_scenarios: usize,
}

impl CampaignHeader {
    pub fn new(name: &str) -> Self {
        Self {
            version: *b"1.00",
            name: name.to_string(),
            num_scenarios: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ScenarioMeta {
    pub(crate) size: usize,
    pub(crate) offset: usize,
    pub(crate) name: String,
    pub(crate) filename: String,
}

pub use read::Campaign;
pub use write::CampaignWriter;
