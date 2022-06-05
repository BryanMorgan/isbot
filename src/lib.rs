//! [![github]](https://github.com/BryanMorgan/isbot)&ensp;[![crates-io]](https://crates.io/crates/isbot)
//!
//! [github]: <https://img.shields.io/badge/github-8da0cb?style=for-the-badge&labelColor=555555&logo=github>
//! [crates-io]: <https://img.shields.io/badge/crates.io-fc8d62?style=for-the-badge&labelColor=555555&logo=rust>
//!
//! Detect bots or crawlers identified by matching a user-agent to a collection of known bot patterns.
//!
//! User-agent patterns are maintained as a single regular expression for fast validation.
//!
//! The default list of user-agent patterns balances a large set of known bots
//! while ensuring real browsers are not falsely identified as bots.
//!
//! # Examples
//!
//! ```
//! use isbot::Bots;
//!
//! let bots = Bots::default();
//! assert!(bots.is_bot("Googlebot-Image/1.0"));
//! assert!(!bots.is_bot("Opera/9.60 (Windows NT 6.0; U; en) Presto/2.1.1"));
//! ```
//!
//! User-agent regular expressions can be added or removed for specific use cases.
//! For example, you could remove the Chrome Lighthouse bot from the list of known bots:
//!
//! ```
//! let mut bots = isbot::Bots::default();
//!
//! // By default Chrome Lighthouse is considered a bot
//! assert!(bots.is_bot("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/78.0.3904.97 Safari/537.36 Chrome-Lighthouse"));
//! // Remove the Chrome Lighthouse regular expression pattern to indicate it is not a bot
//! bots.remove(&["Chrome-Lighthouse"]);
//! assert!(!bots.is_bot("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/78.0.3904.97 Safari/537.36 Chrome-Lighthouse"));
//! ```
//!
//! Or append a new user-agent to detect a custom bot:
//! ```
//! let mut bots = isbot::Bots::default();
//!
//! // Append a new custom bot user-agent regular expression
//! assert!(!bots.is_bot("Mozilla/5.0 (CustomNewTestB0T /1.2)"));
//! bots.append(&[r"CustomNewTestB0T\s/\d\.\d"]);
//! assert!(bots.is_bot("Mozilla/5.0 (CustomNewTestB0T /1.2)"));
//! ```

use regex::Regex;
use std::{collections::HashSet, fmt::Debug};

/// Wrapper struct to maintain bot regular expression patterns
///
/// # Example
///
/// ```
/// use isbot::Bots;
///
/// let bots = Bots::default();
/// ```
#[derive(Debug)]
pub struct Bots {
    user_agent_patterns: HashSet<String>,
    user_agents_regex: Regex,
}

/// Load default bot user-agent regular expressions from a local file, unless the feature is disabled
#[cfg(feature = "include-default-bots")]
const BOT_PATTERNS: &str = include_str!("bot_regex_patterns.txt");

/// Do not load any default user-agent strings into the compiled library if feature is not enabled
#[cfg(not(feature = "include-default-bots"))]
const BOT_PATTERNS: &str = "";

impl Default for Bots {
    /// Constructs a new instance with default user-agent patterns.
    ///
    /// # Example
    ///
    /// ```
    /// use isbot::Bots;
    ///
    /// let bots = Bots::default();
    ///
    /// assert!(bots.is_bot("Googlebot"));
    /// ```
    fn default() -> Self {
        Bots::new(BOT_PATTERNS)
    }
}

impl Bots {
    /// Constructs a new instance with bot user-agent regular expression entries delimited by a newline
    ///
    /// All user-agent regular expressions are converted to lowercase.
    ///
    /// # Example
    ///
    /// ```
    /// use isbot::Bots;
    ///
    /// let custom_user_agent_patterns = r#"
    /// ^Googlebot-Image/
    /// bingpreview/"#;
    /// let bots = Bots::new(custom_user_agent_patterns);
    ///
    /// assert!(bots.is_bot("Googlebot-Image/1.0"));
    /// assert!(bots.is_bot("Mozilla/5.0 (Windows NT 6.1; WOW64) AppleWebKit/534+ (KHTML, like Gecko) BingPreview/1.0b"));
    /// assert!(!bots.is_bot("Googlebot"));
    /// ```
    pub fn new(bot_entries: &str) -> Self {
        let user_agent_patterns = Bots::parse_lines(&bot_entries.to_ascii_lowercase());
        let combined_user_agent_regex = Bots::to_regex(&user_agent_patterns);
        Bots {
            user_agent_patterns,
            user_agents_regex: combined_user_agent_regex,
        }
    }

    /// Returns `true` the user-agent is a known bot.
    ///
    /// The user-agent comparison is done using lowercase.
    ///
    /// # Example
    ///
    /// ```
    /// use isbot::Bots;
    ///
    /// let bots = Bots::default();
    ///
    /// assert!(bots.is_bot("Googlebot/2.1 (+http://www.google.com/bot.html)"));
    /// assert!(!bots.is_bot("Dalvik/2.1.0 (Linux; U; Android 8.0.0; SM-G930F Build/R16NW)"));
    /// ```    
    pub fn is_bot(&self, user_agent: &str) -> bool {
        self.user_agents_regex
            .is_match(&user_agent.to_ascii_lowercase())
    }

    /// Appends bot user-agent regular expressions patterns.
    ///
    /// Duplicates are ignored.
    ///
    /// # Example
    ///
    /// ```
    /// use isbot::Bots;
    ///
    /// let mut bots = Bots::default();
    /// assert!(!bots.is_bot("Mozilla/5.0 (CustomNewTestB0T /1.2)"));
    /// bots.append(&[r"CustomNewTestB0T\s/\d\.\d"]);
    /// assert!(bots.is_bot("Mozilla/5.0 (CustomNewTestB0T /1.2)"));
    ///
    /// let new_bot_patterns = vec!["GoogleMetaverse", "^Special/"];
    /// bots.append(&new_bot_patterns);
    /// assert!(bots.is_bot("Mozilla/5.0 (GoogleMetaverse/1.0)"));
    /// ```
    pub fn append(&mut self, bots: &[&str]) {
        for bot in bots {
            self.user_agent_patterns.insert(bot.to_ascii_lowercase());
        }
        self.update_regex()
    }

    /// Removes bot user-agent regular expressions.
    ///
    /// # Example
    ///
    /// ```
    /// use isbot::Bots;
    ///
    /// let mut bots = Bots::default();
    ///
    ///
    /// assert!(bots.is_bot("Chrome-Lighthouse"));
    /// bots.remove(&["Chrome-Lighthouse"]);
    /// assert!(!bots.is_bot("Chrome-Lighthouse"));
    ///
    /// let bot_patterns_to_remove = vec!["bingpreview/", "Google Favicon"];
    /// bots.remove(&bot_patterns_to_remove);
    /// assert!(!bots.is_bot("Mozilla/5.0 (Windows NT 6.1; WOW64) AppleWebKit/534+ (KHTML, like Gecko) BingPreview/1.0b"));
    /// assert!(!bots.is_bot("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/49.0.2623.75 Safari/537.36 Google Favicon"));
    /// ```
    pub fn remove(&mut self, bots: &[&str]) {
        for bot in bots {
            self.user_agent_patterns.remove(&bot.to_ascii_lowercase());
        }
        self.update_regex()
    }

    fn update_regex(&mut self) {
        self.user_agents_regex = Bots::to_regex(&self.user_agent_patterns)
    }

    fn parse_lines(bot_regex_entries: &str) -> HashSet<String> {
        HashSet::from_iter(
            bot_regex_entries
                .lines()
                .filter(|l| !l.trim().is_empty())
                .map(ToString::to_string),
        )
    }

    fn to_regex(regex_entries: &HashSet<String>) -> Regex {
        let pattern = regex_entries
            .iter()
            .cloned()
            .collect::<Vec<String>>()
            .join("|");

        if pattern.is_empty() {
            return Regex::new("^$").unwrap();
        }

        Regex::new(&pattern).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use crate::Bots;

    static GOOD_BOTS: [&str; 7] = [
        "Googlebot",
        "Mozilla/5.0 (compatible; Googlebot/2.1; +http://www.google.com/bot.html)",
        "Mozilla/5.0 (compatible; Yahoo! Slurp; http://help.yahoo.com/help/us/ysearch/slurp)",
        "Mozilla/5.0 (Linux; Android 6.0.1; Nexus 5X Build/MMB29P) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/41.0.2272.96 Mobile Safari/537.36 (compatible; Googlebot/2.1; +http://www.google.com/bot.html)",
        "Mozilla/5.0 (compatible; Bingbot/2.0; +http://www.bing.com/bingbot.htm)",
        "DuckDuckBot/1.0; (+http://duckduckgo.com/duckduckbot.html)",
        "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/78.0.3904.97 Safari/537.36 Chrome-Lighthouse"
    ];

    static NOT_BOTS: [&str; 6] = [
        "",
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/58.0.3029.110 Safari/537.36",
        "Mozilla/4.0 (compatible; MSIE 8.0; Windows NT 5.1; Trident/4.0; .NET CLR 1.1.4322; .NET CLR 2.0.50727; .NET CLR 3.0.4506.2152; .NET CLR 3.5.30729)",
        "Mozilla/5.0 (iPhone; CPU iPhone OS 10_3_1 like Mac OS X) AppleWebKit/603.1.30 (KHTML, like Gecko) Version/10.0 Mobile/14E304 Safari/602.1",
        "Mozilla/5.0 (Linux; Android 5.0; SAMSUNG SM-N900 Build/LRX21V) AppleWebKit/537.36 (KHTML, like Gecko) SamsungBrowser/2.1 Chrome/34.0.1847.76 Mobile Safari/537.36",
        "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/95.0.4638.54 Safari/537.36",
    ];

    #[test]
    fn good_bots() {
        let bots = Bots::default();
        for bot in GOOD_BOTS {
            assert!(bots.is_bot(bot), "Invalid bot: '{}'", bot);
        }
    }

    #[test]
    fn not_bots() {
        let bots = Bots::default();
        for bot in NOT_BOTS {
            assert!(!bots.is_bot(bot), "Is a bot{}", bot);
        }
    }

    #[test]
    fn custom_user_agent_patterns() {
        let custom_user_agent_patterns = "\
            ^Simplebot\n\
            anything\\s+bot\n\
            Numerical\\d{4}\\.\\d{4}\\.\\d{4}\\.\\d{4}";
        let bots = Bots::new(custom_user_agent_patterns);
        assert!(!bots.is_bot("InvalidBot"));
        assert!(!bots.is_bot("Googlebot"));
        assert!(bots.is_bot("Simplebot/1.2"));
        assert!(!bots.is_bot(" Simplebot/1.2"));
        assert!(bots.is_bot("Anything  Bot"));
        assert!(!bots.is_bot("AnythingBot"));
        assert!(bots.is_bot("numerical1101.2001.3987.4781"));
        assert!(!bots.is_bot("numerical1.2.3.4"));
    }

    #[test]
    fn empty_user_agent_patterns() {
        let empty_user_agent_patterns = "";
        let bots = Bots::new(empty_user_agent_patterns);
        assert!(bots.is_bot(""));
        assert!(!bots.is_bot("1"));
        assert!(!bots.is_bot("Googlebot"));
    }

    #[test]
    fn single_user_agent_patterns() {
        let single_user_agent_patterns = "me";
        let bots = Bots::new(single_user_agent_patterns);
        assert!(!bots.is_bot(""));
        assert!(!bots.is_bot("M"));
        assert!(bots.is_bot("Me"));
        assert!(!bots.is_bot("Googlebot"));
    }
    #[test]
    fn add_pattern() {
        let mut bots = Bots::default();
        assert!(!bots.is_bot("Mozilla/5.0 (FancyNewTestB0T /1.2)"));
        bots.append(&[r"FancyNewTestB0T\s/\d\.\d"]);
        assert!(bots.is_bot("Mozilla/5.0 (FancyNewTestB0T /1.2)"));
    }

    #[test]
    fn add_multiple_patterns() {
        let mut bots = Bots::default();
        assert!(!bots.is_bot("Mozilla/5.0 (FancyNewTestB0T /1.2)"));
        assert!(!bots.is_bot("Special/1.0"));
        assert!(!bots.is_bot("GoogleMetaverse/2.1 (experimental)"));

        let new_bot_patterns = vec!["FancyNewTestB0T", "^GoogleMetaverse", "^Special/"];
        bots.append(&new_bot_patterns);

        assert!(bots.is_bot("Mozilla/5.0 (FancyNewTestB0T /1.2)"));
        assert!(bots.is_bot("Special/1.0"));
        assert!(bots.is_bot("GoogleMetaverse/2.1 (experimental)"));
    }

    #[test]
    fn remove_pattern() {
        let mut bots = Bots::default();
        assert!(bots.is_bot("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/78.0.3904.97 Safari/537.36 Chrome-Lighthouse"));
        bots.remove(&["Chrome-Lighthouse"]);
        assert!(!bots.is_bot("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/78.0.3904.97 Safari/537.36 Chrome-Lighthouse"));
        assert!(!bots.is_bot("Chrome-Lighthouse"));
        assert!(bots.is_bot("Mozilla/5.0 (Windows NT 10.0; Win64; x64) adbeat.com/policy AppleWebKit/537.36 (KHTML, like Gecko) Chrome/73.0.3683.86 Safari/537.36"));
    }

    #[test]
    fn remove_multiple_patterns() {
        let mut bots = Bots::default();
        assert!(bots.is_bot("Datadog Agent/5.10.1"));
        assert!(bots.is_bot("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/78.0.3904.97 Safari/537.36 Chrome-Lighthouse"));
        assert!(bots.is_bot("Mozilla/5.0 (Java) outbrain"));
        assert!(bots.is_bot("Mozilla/5.0 (compatible; Google-Site-Verification/1.0)"));

        let bot_patterns_to_remove =
            vec!["datadog agent", "Chrome-Lighthouse", "outbrain", "google-"];
        bots.remove(&bot_patterns_to_remove);

        assert!(!bots.is_bot("Datadog Agent/5.10.1"));
        assert!(!bots.is_bot("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/78.0.3904.97 Safari/537.36 Chrome-Lighthouse"));
        assert!(!bots.is_bot("Mozilla/5.0 (Java) outbrain"));
        assert!(!bots.is_bot("Mozilla/5.0 (compatible; Google-Site-Verification/1.0)"));
    }
}
