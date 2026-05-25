#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PackType {
    ClientResources,
    ServerData,
}

impl PackType {
    pub fn directory(&self) -> &'static str {
        match self {
            PackType::ClientResources => "assets",
            PackType::ServerData => "data",
        }
    }
}
