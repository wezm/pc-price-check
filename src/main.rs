use std::path::Path;
use std::{env, fs};

use anyhow::{bail, Context};
use pc_price_check::{Component, ComponentType, PartsList, SearchResult};
use scraper::{Html, Selector};
use url::Url;

const CONFIG: &str = "parts.toml";

fn main() -> anyhow::Result<()> {
    // NOTE(unwrap): Safe as selector to known to be valid
    let links = Selector::parse("a[href*='/cgi-bin/redirect.cgi'][alt]").unwrap();
    let parts = load_parts_list(Path::new(CONFIG))?;
    let mut components = parts.parts;
    components.sort_by(|a, b| {
        a.component_type
            .cmp(&b.component_type)
            .then(a.query.cmp(&b.query))
    });

    if let Some("-l") = env::args_os()
        .skip(1)
        .next()
        .as_deref()
        .and_then(|s| s.to_str())
    {
        list_components(&components);
        return Ok(());
    }

    let mut agent = ureq::agent();
    let mut current_component = None;
    for (i, item) in components.iter().enumerate() {
        let Component {
            component_type: component,
            query: q,
            price: reference,
        } = item;

        if current_component != Some(component) {
            current_component = Some(component);
            println!(
                "{}\x1B[35m{}:\x1B[m",
                if i == 0 { "" } else { "\n" },
                component
            );
        }

        // We do them in sequence because StaticICE limits concurrent requests to 3
        let res = search(&mut agent, component, q)?;
        let doc = Html::parse_document(&res.page);

        match doc.select(&links).next() {
            Some(el) => {
                let price = el.text().collect::<String>();
                let price = if let Some(price) = price.strip_prefix('$') {
                    price
                        .parse::<f32>()
                        .with_context(|| format!("Unable to parse {}", price))?
                } else {
                    bail!("Price does not start with $");
                };
                let diff = price - reference;
                let diff_fmt = match diff {
                    _ if diff < 0. => format!(" \x1B[32m-${:.02}\x1B[m", diff.abs()), // green
                    _ if diff > 0. => format!(" \x1B[31m+${:.02}\x1B[m ", diff),      // red
                    _ => String::new(),                                               // zero
                };
                let reffmt = if diff != 0.0 {
                    format!("\t\x1B[2m(${:.02})\x1B[m", reference)
                } else {
                    String::new()
                };
                println!("    {}: ${:.02}{}{}", q, price, diff_fmt, reffmt)
            }
            None => println!("{}: No match: {}", res.component, doc.html()),
        }
    }

    Ok(())
}

fn search<'a>(
    agent: &mut ureq::Agent,
    component: &'a ComponentType,
    q: &str,
) -> anyhow::Result<SearchResult<'a>> {
    let url = Url::parse_with_params(
        "https://www.staticice.com.au/cgi-bin/search.cgi?spos=3",
        &[("q", q)],
    )?;
    let page = agent.get(url.as_str()).call()?.into_string()?;
    Ok(SearchResult { component, page })
}

fn load_parts_list(path: &Path) -> anyhow::Result<PartsList> {
    let text = fs::read_to_string(path).with_context(|| format!("unable to read {CONFIG}"))?;
    let list = toml::from_str::<PartsList>(&text).with_context(|| "unable to parse parts list")?;
    Ok(list)
}

fn list_components(components: &[Component]) {
    for item in components {
        let component = &item.component_type;
        let name = &item.query;
        let price = item.price;

        println!("{}: {} - ${:.02}", component, name, price);
    }
}
