use isbot::Bots;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

// Test downloaded browser patterns from https://github.com/ua-parser/uap-core
// and https://github.com/omrilotan/isbot
const UA_PARSER_BROWSERS: &str = "ua-parser-browsers.json";
const OMRILOTAN_BROWSERS_FILE: &str = "omrilotan-browsers.json";

#[test]
fn test_ua_core_browsers() {
    validate_browsers(UA_PARSER_BROWSERS)
}

#[test]
fn test_omrilotan_browsers() {
    validate_browsers(OMRILOTAN_BROWSERS_FILE)
}

fn validate_browsers(filename: &str) {
    let bots = Bots::default();

    for user_agent in get_json(filename) {
        assert_eq!(
            bots.is_bot(&user_agent),
            false,
            "User-agent is a bot, not a browser: {}",
            user_agent
        );
    }
}

fn get_json(filename: &str) -> Vec<String> {
    let path = Path::new("fixtures").join(filename);
    let file = File::open(&path).unwrap_or_else(|_| panic!("Unable to open file: {:?}", path));
    let reader = BufReader::new(file);
    serde_json::from_reader(reader).expect("Could not parse JSON")
}
