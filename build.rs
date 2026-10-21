use headless_chrome::{Browser, LaunchOptions};
use regex::Regex;
use scraper::{Html, Selector};
use std::{
    error::Error,
    fs::{self, File},
    io::Write,
    path::Path,
    time::Duration,
};

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=FORCE_RESCRAPE");

    let output_dir = Path::new("src");
    let output_path = output_dir.join("schedule_data.rs");

    let force = std::env::var("FORCE_RESCRAPE")
        .map(|v| !v.is_empty() && v != "0")
        .unwrap_or(false);

    let has_data = fs::read_to_string(&output_path)
        .map(|c| c.contains("Interval {"))
        .unwrap_or(false);

    if has_data && !force {
        println!(
            "cargo:warning=schedule_data.rs already populated; skipping scrape. \
             Run `FORCE_RESCRAPE=1 trunk build` (or cargo build) to refresh it."
        );
        return;
    }

    if !output_path.exists() {
        let _ = fs::write(
            &output_path,
            "use crate::schedule::Session;\npub const SCHEDULE: &[Session] = &[];\n",
        );
    }

    match scrape_schedule(&output_path) {
        Ok(count) => {
            println!("cargo:warning=Scraped {count} sessions into schedule_data.rs.");
        }
        Err(e) => {
            // Don't fail the build offline / when Chrome is missing / if the
            // site markup changed — keep whatever data is already there.
            println!(
                "cargo:warning=Schedule scrape failed: {e}. \
                 Keeping existing schedule_data.rs."
            );
        }
    }
}

fn scrape_schedule(output_path: &Path) -> Result<usize, Box<dyn Error>> {
    let url = "https://start-to-run.be/loopschema-5-km-voor-beginners";
    println!("cargo:warning=Launching headless browser to harvest schema from {url} ...");

    // Initialize headless browser engine
    let options = LaunchOptions::default();
    let browser = Browser::new(options)?;
    let tab = browser.new_tab()?;

    tab.navigate_to(url)?;
    tab.wait_until_navigated()?;

    // Wait up to 30 seconds for the React app to populate the table rows.
    tab.wait_for_element_with_custom_timeout("tr", Duration::from_secs(30))?;

    let fully_rendered_html = tab.get_content()?;
    let document = Html::parse_document(&fully_rendered_html);

    let row_selector = Selector::parse("tr").unwrap();
    let cell_selector = Selector::parse("td").unwrap();
    let interval_selector = Selector::parse("div[title]").unwrap();

    let session_re = Regex::new(r"(?i)week\s*(\d+)\s*sessie\s*(\d+)")?;
    let duration_re = Regex::new(r"(?i)(\d+)\s*min")?;

    let mut rust_code = String::new();
    rust_code.push_str("// =========================================================\n");
    rust_code.push_str("// GENERATED AUTOMATICALLY BY BUILD.RS VIA SCRAPER. DO NOT EDIT.\n");
    rust_code.push_str("// =========================================================\n\n");
    rust_code.push_str("use crate::schedule::{Session, Interval, IntervalKind};\n\n");
    rust_code.push_str("pub const SCHEDULE: &[Session] = &[\n");
    rust_code.push_str(
        r#"    Session {
        week: 1,
        session: 0,
        intervals: &[
            Interval { kind: IntervalKind::Run, duration_secs: 10 },
            Interval { kind: IntervalKind::Walk, duration_secs: 10 },
            Interval { kind: IntervalKind::Run, duration_secs: 10 },
            Interval { kind: IntervalKind::Walk, duration_secs: 10 },
        ],
    },
"#,
    );

    let mut entries_count = 0;

    for row in document.select(&row_selector) {
        let cells: Vec<_> = row.select(&cell_selector).collect();
        if cells.len() < 2 {
            continue;
        }

        let cell_text = cells[0].text().collect::<String>();
        let (week, session) = match session_re.captures(&cell_text) {
            Some(caps) => (caps[1].parse::<u32>()?, caps[2].parse::<u32>()?),
            None => continue,
        };

        let mut intervals_code = String::new();
        let mut has_intervals = false;

        for div in cells[1].select(&interval_selector) {
            if let Some(title_attr) = div.value().attr("title") {
                let kind_str = if title_attr.contains("Lopen") {
                    "IntervalKind::Run"
                } else if title_attr.contains("Wandelen") {
                    "IntervalKind::Walk"
                } else {
                    continue;
                };

                if let Some(caps) = duration_re.captures(title_attr) {
                    let minutes = caps[1].parse::<u32>()?;
                    let seconds = minutes * 60;
                    intervals_code.push_str(&format!(
                        "            Interval {{ kind: {}, duration_secs: {} }},\n",
                        kind_str, seconds
                    ));
                    has_intervals = true;
                }
            }
        }

        if has_intervals {
            rust_code.push_str(&format!(
                "    Session {{\n        week: {},\n        session: {},\n        intervals: &[\n{}",
                week, session, intervals_code
            ));
            rust_code.push_str("        ],\n    },\n");
            entries_count += 1;
        }
    }

    rust_code.push_str("];\n");

    // Only overwrite if we actually parsed something, so a partial/empty scrape
    // never clobbers good committed data.
    if entries_count == 0 {
        return Err("no sessions parsed from page".into());
    }

    let mut file = File::create(output_path)?;
    file.write_all(rust_code.as_bytes())?;

    Ok(entries_count)
}
