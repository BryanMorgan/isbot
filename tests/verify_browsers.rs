use isbot::Bots;
use std::fs::File;
use std::io::{prelude::*, BufReader};
use std::path::Path;

#[test]
fn test_fixture_browsers() {
    let bots = Bots::default();

    let path = Path::new("fixtures").join("browsers.txt");
    let file = File::open(&path).unwrap_or_else(|_| panic!("Unable to open file: {:?}", path));
    let reader = BufReader::new(file);
    for user_agent in reader.lines().flatten() {
        assert!(
            !bots.is_bot(&user_agent),
            "User-agent is a bot, not a browser: {}",
            user_agent
        );
    }
}
