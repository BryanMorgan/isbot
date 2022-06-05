use isbot::Bots;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

// Test downloaded bot patterns from https://myip.ms/ and
// https://github.com/ua-parser/uap-core
const MYIP_MS_BOTS_FILE: &str = "myip-ms-live-bots.json";
const UA_PARSER_BOTS: &str = "ua-parser-bots.json";

#[test]
fn test_myip_ms_bots() {
    validate_bots(MYIP_MS_BOTS_FILE)
}

#[test]
fn test_ua_parser_bots() {
    validate_bots(UA_PARSER_BOTS)
}

fn validate_bots(filename: &str) {
    let bots = Bots::default();

    for user_agent in get_json(filename) {
        // myip.ms incorrectly identifies CUBOT as a bot
        if !user_agent.contains(" CUBOT") {
            assert!(
                bots.is_bot(&user_agent),
                "User-agent is not a bot: {}",
                user_agent
            );
        }
    }
}

fn get_json(filename: &str) -> Vec<String> {
    let path = Path::new("fixtures").join(filename);
    let file = File::open(&path).unwrap_or_else(|_| panic!("Unable to open file: {:?}", path));
    let reader = BufReader::new(file);
    serde_json::from_reader(reader).expect("Could not parse JSON")
}
