use std::fmt::{self, Formatter};

use serde::Deserialize;

#[derive(Deserialize, Debug, Eq, PartialEq, Ord, PartialOrd, Clone)]
pub enum ComponentType {
    CPUCooling,
    Case,
    Cpu,
    Graphics,
    Keyboard,
    Memory,
    Monitor,
    Motherboard,
    PowerSupply,
    Network,
    Ssd,
    Hdd,
    #[serde(untagged)]
    Other(String),
}

#[derive(Deserialize)]
pub struct Component {
    pub component_type: ComponentType,
    pub query: String,
    pub price: f32,
}

#[derive(Deserialize)]
pub struct PartsList {
    pub parts: Vec<Component>,
}

pub struct SearchResult<'a> {
    pub component: &'a ComponentType,
    pub page: String,
}

impl fmt::Display for ComponentType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ComponentType::CPUCooling => f.write_str("CPU Cooler"),
            ComponentType::Case => f.write_str("Case"),
            ComponentType::Cpu => f.write_str("CPU"),
            ComponentType::Graphics => f.write_str("Graphics"),
            ComponentType::Keyboard => f.write_str("Keyboard"),
            ComponentType::Memory => f.write_str("Memory"),
            ComponentType::Monitor => f.write_str("Monitor"),
            ComponentType::Motherboard => f.write_str("Motherboard"),
            ComponentType::PowerSupply => f.write_str("Power Supply"),
            ComponentType::Network => f.write_str("Network"),
            ComponentType::Ssd => f.write_str("Primary SSD"),
            ComponentType::Hdd => f.write_str("Hard Drive"),
            ComponentType::Other(other) => f.write_str(other),
        }
    }
}
