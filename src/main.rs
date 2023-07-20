use scraper::{Html, Selector};
use std::cmp::Ordering;
use std::env;
use std::fmt::{self, Formatter};
use std::path::Path;
use url::Url;
use serde_derive::Deserialize;

#[derive(Deserialize, Debug, Eq, PartialEq, Ord, PartialOrd, Copy, Clone)]
enum ComponentType {
    // Ssd2,
    CPUCooling,
    Case,
    Cpu,
    Graphics,
    Keyboard,
    Memory,
    Motherboard,
    PowerSupply,
    Network,
    Ssd,
    Hdd,
    Other,
}

#[derive(Deserialize, Eq)]
struct Component {
    component_type: ComponentType,
    query_string: String,
    price: i32,
}

#[derive(Deserialize, Eq, PartialEq)]
struct PartsList {
    parts: Vec<Component>,
}

struct SearchResult {
    component: ComponentType,
    page: String,
}

fn load_parts_list(path: &Path) -> PartsList {
    let text = std::fs::read_to_string(path).unwrap();
    let parts: Result::<PartsList, _> = toml::from_str(&text);

    parts.unwrap()
}

fn main() {

    let links = Selector::parse("a[href*='/cgi-bin/redirect.cgi'][alt]").unwrap();

    let parts = load_parts_list(Path::new("parts.toml"));
    let mut components = parts.parts;
    components.sort();

    if let Some("-l") = env::args_os()
        .skip(1)
        .next()
        .as_deref()
        .and_then(|s| s.to_str())
    {
        return list_components(&components);
    }

    let mut agent = ureq::agent();

    let mut current_type: ComponentType = components[0].component_type;
    println!("{}:", current_type);

    for item in components {
        let component = item.component_type;
        let q = item.query_string;
        let reference = item.price;

        if current_type != component {
            current_type = component;
            println!("{}:", current_type);
        }

        // We do them in sequence because StaticICE limits concurrent requests to 3
        let res = search(&mut agent, component, &q);
        let doc = Html::parse_document(&res.page);

        match doc.select(&links).next() {
            Some(el) => {
                let price = el.text().collect::<String>();
                let price = if price.starts_with('$') {
                    let p = price[1..]
                        .chars()
                        .filter(|c| c.is_ascii_digit())
                        .collect::<String>();
                    p.parse::<i32>()
                        .unwrap_or_else(|err| panic!("Unable to parse {}: {}", p, err))
                } else {
                    panic!("Price does not start with $");
                };
                let diff = price - reference;
                let diff = match diff {
                    _ if diff < 0 => format!(" \x1B[32m-${:.02}\x1B[m", diff.abs() as f64 / 100.), // green
                    _ if diff > 0 => format!(" \x1B[31m+${:.02}\x1B[m ", diff as f64 / 100.), // red
                    _ => String::new(), // zero
                };
                println!("    {}: ${:.02}{}", q, price as f64 / 100., diff)
            }
            None => println!("{}: No match: {}", res.component, doc.html()),
        }
    }
}

fn search(agent: &mut ureq::Agent, component: ComponentType, q: &str) -> SearchResult {
    let url = Url::parse_with_params(
        "https://www.staticice.com.au/cgi-bin/search.cgi?spos=3",
        &[("q", q)],
    )
    .unwrap();
    match agent.get(url.as_str()).call() {
        Ok(resp) => {
            let page = resp.into_string().unwrap();
            SearchResult { component, page }
        }
        Err(err) => {
            panic!("Unsuccessful request ({}) {}", url, err)
        }
    }
}

impl fmt::Display for ComponentType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            // Component::Ssd2 => f.write_str("Aux SSD"),
            ComponentType::CPUCooling => f.write_str("CPU Cooler"),
            ComponentType::Case => f.write_str("Case"),
            ComponentType::Cpu => f.write_str("CPU"),
            ComponentType::Graphics => f.write_str("Graphics"),
            ComponentType::Keyboard => f.write_str("Keyboard"),
            ComponentType::Memory => f.write_str("Memory"),
            ComponentType::Motherboard => f.write_str("Motherboard"),
            ComponentType::PowerSupply => f.write_str("Power Supply"),
            ComponentType::Network => f.write_str("Network"),
            ComponentType::Ssd => f.write_str("Primary SSD"),
            ComponentType::Hdd => f.write_str("Hard Drive"),
            ComponentType::Other => f.write_str("Other"),
        }
    }
}

impl Ord for Component {
    fn cmp(&self, other: &Self) -> Ordering {
        self.component_type.cmp(&other.component_type).then(self.query_string.cmp(&other.query_string))
    }
}

impl PartialOrd for Component {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Component {
    fn eq(&self, other: &Self) -> bool {
        self.component_type == other.component_type
    }
}

fn list_components(components: &[Component]) {
    for item in components {
        let component = item.component_type;
        let name = &item.query_string;
        let price = item.price;

        println!("{}: {} - ${:.02}", component, name, price as f64 / 100.);
    }
}