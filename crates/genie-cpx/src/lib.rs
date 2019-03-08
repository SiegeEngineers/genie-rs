mod read;
mod write;

#[derive(Debug, Clone)]
pub struct CampaignHeader {
    pub(crate) version: f32,
    pub(crate) name: String,
    pub(crate) num_scenarios: usize,
}

impl CampaignHeader {
    pub fn new(name: &str) -> Self {
        Self {
            version: 2.0,
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

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
