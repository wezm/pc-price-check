use scraper::{Html, Selector};
use std::env;
use std::fmt::{self, Formatter};
use url::Url;

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
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
    Ssd,
}

struct SearchResult {
    component: ComponentType,
    page: String,
}

fn main() {
    use ComponentType::*;

    let links = Selector::parse("a[href*='/cgi-bin/redirect.cgi'][alt]").unwrap();

    let components = [
        (Cpu, "AMD Ryzen 9 7950X", 860_00i32),
        (Motherboard, "Gigabyte X670 AORUS ELITE AX", 450_00),
        (Graphics, "Gigabyte Radeon RX 6700 XT EAGLE", 478_00),
        (Memory, "Corsair CMK32GX5M2D6000Z36", 175_00),
        (Ssd, "Crucial CT1000T700SSD3", 304_88),
        // (Ssd2, "CT2000P5PSSD8"),
        (Case, "Fractal FD-C-TOR1A-03", 289_00),
        (PowerSupply, "Corsair CP-9020199-AU", 149_00),
        (CPUCooling, "Noctua NH-D15 CPU Cooler -NH-D15S", 146_00),
        (Keyboard, "KBKCQ3N3BROWN", 279_00),
    ];

    if let Some("-l") = env::args_os()
        .skip(1)
        .next()
        .as_deref()
        .and_then(|s| s.to_str())
    {
        return list_components(&components);
    }

    let mut agent = ureq::agent();

    for (component, q, reference) in components {
        // We do them in sequence because StaticICE limits concurrent requests to 3
        let res = search(&mut agent, component, q);
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
                println!("{}: ${:.02}{}", res.component, price as f64 / 100., diff)
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
            ComponentType::Ssd => f.write_str("Primary SSD"),
        }
    }
}

fn list_components(components: &[(ComponentType, &'static str, i32)]) {
    for (component, name, price) in components {
        println!("{}: {} - ${:.02}", component, name, *price as f64 / 100.);
    }
}
