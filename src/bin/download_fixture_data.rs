use regex::Regex;
use std::error::Error;
use std::fs::{create_dir_all, File};
use std::io::{prelude::*, BufWriter};
use std::path::Path;
use std::thread;
use std::time::Duration;

use ureq::Agent;
use yaml_rust::yaml;

type Result<T> = std::result::Result<T, Box<dyn Error + Send + Sync>>;
type DownloadTask = fn(Agent) -> Result<()>;

const MYIP_MS_BOTS: &str = "myip-ms-live-bots";
const UA_PARSER_BOTS: &str = "ua-parser-bots";
const UA_PARSER_BROWSERS: &str = "ua-parser-browsers";
const OMRILOTAN_BROWSERS: &str = "omrilotan-browsers";

const MYIP_MS_URL: &str = "https://myip.ms/files/bots/live_webcrawlers.txt";
const OMRILOTAN_BROWSERS_URL: &str =
    "https://raw.githubusercontent.com/omrilotan/isbot/main/fixtures/browsers.yml";
const UA_PARSER_BROWSERS_URL: &str =
    "https://raw.githubusercontent.com/ua-parser/uap-core/master/tests/test_device.yaml";

/// Executable to download test fixture data from multiple sources.
/// Spawns multiple threads to download, parse, and output fixture data.
/// All output is added to the `fixtures/` directory and used in integration tests.
///
/// # To run
///
/// ```
/// cargo run --bin download_fixture_data --features="download-fixture-data"
/// ```
fn main() -> Result<()> {
    let mut threads = vec![];
    let tasks: Vec<DownloadTask> = vec![download_ua_parser, download_myips_ms, download_omrilotan];

    for task in tasks {
        let agent: Agent = ureq::AgentBuilder::new()
            .timeout_read(Duration::from_secs(10))
            .build();
        threads.push(thread::spawn(move || task(agent)));
    }

    for thread in threads {
        let _ = thread
            .join()
            .unwrap()
            .map_err(|e| println!("Failed: {}", e));
    }

    Ok(())
}

fn download_ua_parser(agent: Agent) -> Result<()> {
    println!("[{:<18}] start download of YAML file", UA_PARSER_BROWSERS);

    let response: String = agent.get(UA_PARSER_BROWSERS_URL).call()?.into_string()?;
    let mut browsers: Vec<String> = vec![];
    let mut bots: Vec<String> = vec![];

    let docs = yaml::YamlLoader::load_from_str(&response).expect("Could not load YAML from string");
    let empty = yaml::Yaml::from_str("");
    for doc in docs[0].as_hash().expect("Invalid YAML: expected array") {
        for crawler_entry in doc.1.as_vec().expect("Not an array") {
            if let yaml::Yaml::Hash(hash_node) = &crawler_entry {
                let user_agent = hash_node
                    .get(&yaml::Yaml::from_str("user_agent_string"))
                    .unwrap_or(&empty)
                    .as_str()
                    .unwrap_or("")
                    .to_string();
                let user_agent_lowercase = user_agent.to_ascii_lowercase();
                let family: &str = hash_node
                    .get(&yaml::Yaml::from_str("family"))
                    .unwrap_or(&empty)
                    .as_str()
                    .unwrap_or("");

                if family == "Spider" {
                    bots.push(user_agent);
                } else if family != "Other"
                    && !user_agent_lowercase.contains("spider")
                    && !user_agent_lowercase.contains("http://")
                {
                    browsers.push(user_agent);
                }
            }
        }
    }

    write_crawlers_to_json_file(&mut browsers, UA_PARSER_BROWSERS)?;
    write_crawlers_to_json_file(&mut bots, UA_PARSER_BOTS)?;

    Ok(())
}

fn download_omrilotan(agent: Agent) -> Result<()> {
    println!("[{:<18}] start download of YAML file", OMRILOTAN_BROWSERS);

    let response: String = agent.get(OMRILOTAN_BROWSERS_URL).call()?.into_string()?;
    let mut values: Vec<String> = vec![];
    let docs = yaml::YamlLoader::load_from_str(&response).expect("Could not load YAML from string");
    for doc in docs[0].as_hash().expect("Invalid YAML: expected array") {
        for value in doc.1.as_vec().expect("Invalid YAML: Not an array") {
            let crawler = value.as_str().expect("Value is not a string");
            values.push(crawler.into());
        }
    }

    write_crawlers_to_json_file(&mut values, OMRILOTAN_BROWSERS)?;

    Ok(())
}

fn download_myips_ms(agent: Agent) -> Result<()> {
    println!("[{:<18}] start download of TEXT file", MYIP_MS_BOTS);
    let line_regex = Regex::new("^#.+records - (.+)?").unwrap();

    let mut values = agent
        .get(MYIP_MS_URL)
        .call()?
        .into_string()?
        .lines()
        .filter_map(|s| line_regex.captures(s)?.get(1))
        .map(|m| m.as_str().to_string())
        .collect::<Vec<String>>();

    write_crawlers_to_json_file(&mut values, MYIP_MS_BOTS)?;

    Ok(())
}

fn write_crawlers_to_json_file(crawlers: &mut [String], name: &str) -> Result<()> {
    crawlers.sort_unstable();
    let json_string = serde_json::to_string_pretty(&crawlers)?;
    let mut fixtures_path = Path::new(".").join("fixtures");
    if !fixtures_path.is_dir() {
        create_dir_all(&fixtures_path)?;
    }

    fixtures_path.push(name);
    fixtures_path.set_extension("json");
    let file = File::create(&fixtures_path)
        .unwrap_or_else(|_| panic!("Unable to create file: {:?}", fixtures_path));
    let mut writer = BufWriter::new(file);
    writer.write_all(json_string.as_bytes())?;

    println!(
        "[{:<18}] saved {} user-agents to {:?}",
        name,
        crawlers.len(),
        fixtures_path
    );

    Ok(())
}
