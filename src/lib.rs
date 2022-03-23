use regex::Regex;
use std::{collections::HashSet, fmt::Debug};

#[derive(Debug)]

/// A collection of user-agent patterns used to detect known bots.
///
/// User-agent patterns are maintained as a single regular expression for fast validation.
///
/// The default list of user-agent patterns balances a large set of known bots/crawlers
/// while ensuring devices and browsers are not falsely identified as bots.
///
/// # Examples
///
/// ```
/// use isbot::Bots;
///
/// let bots = Bots::default();
/// assert_eq!(bots.is_bot("Googlebot-Image/1.0"), true);
/// assert_eq!(bots.is_bot("Opera/9.60 (Windows NT 6.0; U; en) Presto/2.1.1"), false);
/// ```
///
/// User-agent regular expressions can be added or removed for specific use cases.
/// For example, you could remove the Chrome Lighthouse bot from the list of known bots:
///
/// ```
/// let mut bots = isbot::Bots::default();
///
/// // By default Chrome Lighthouse is considered a bot
/// assert_eq!(bots.is_bot("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/78.0.3904.97 Safari/537.36 Chrome-Lighthouse"), true);
/// // Remove the Chrome Lighthouse regular expression pattern to indicate it is not a bot
/// bots.remove(&["Chrome-Lighthouse"]);
/// assert_eq!(bots.is_bot("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/78.0.3904.97 Safari/537.36 Chrome-Lighthouse"), false);
///
/// // Append a new custom bot user-agent regular expression
/// assert_eq!(bots.is_bot("Mozilla/5.0 (CustomNewTestB0T /1.2)"), false);
/// bots.append(&[r"CustomNewTestB0T\s/\d\.\d"]);
/// assert_eq!(bots.is_bot("Mozilla/5.0 (CustomNewTestB0T /1.2)"), true);
///
pub struct Bots {
    user_agent_patterns: HashSet<String>,
    combined_user_agent_regex: Regex,
}

/// Load default bot user-agent regular expressions from a local file, unless the feature is disabled
#[cfg(feature = "include-default-bots")]
const BOT_REGEX_LIST: &str = include_str!("bot_patterns.txt");

/// Do not load any default user-agent strings into the compiled library if feature is not enabled
#[cfg(not(feature = "include-default-bots"))]
const BOT_REGEX_LIST: &str = "";

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
    /// assert_eq!(bots.is_bot("Googlebot"), true);
    /// ```
    fn default() -> Self {
        Bots::new(BOT_REGEX_LIST)
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
    /// assert_eq!(bots.is_bot("Googlebot-Image/1.0"), true);
    /// assert_eq!(bots.is_bot("Mozilla/5.0 (Windows NT 6.1; WOW64) AppleWebKit/534+ (KHTML, like Gecko) BingPreview/1.0b"), true);
    /// assert_eq!(bots.is_bot("Googlebot"), false);
    /// ```
    pub fn new(bot_entries: &str) -> Self {
        let user_agent_patterns = Bots::parse_lines(&bot_entries.to_ascii_lowercase());
        let combined_user_agent_regex = Bots::to_regex(&user_agent_patterns);
        Bots {
            user_agent_patterns,
            combined_user_agent_regex,
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
    /// assert_eq!(bots.is_bot("Googlebot/2.1 (+http://www.google.com/bot.html)"), true);
    /// assert_eq!(bots.is_bot("Dalvik/2.1.0 (Linux; U; Android 8.0.0; SM-G930F Build/R16NW)"), false);
    /// ```    
    pub fn is_bot(&self, user_agent: &str) -> bool {
        self.combined_user_agent_regex
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
    /// assert_eq!(bots.is_bot("Mozilla/5.0 (CustomNewTestB0T /1.2)"), false);
    /// bots.append(&[r"CustomNewTestB0T\s/\d\.\d"]);
    /// assert_eq!(bots.is_bot("Mozilla/5.0 (CustomNewTestB0T /1.2)"), true);
    ///
    /// let new_bot_patterns = vec!["GoogleMetaverse", "^Special/"];
    /// bots.append(&new_bot_patterns);
    /// assert_eq!(bots.is_bot("Mozilla/5.0 (GoogleMetaverse/1.0)"), true);
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
    /// assert_eq!(bots.is_bot("Chrome-Lighthouse"), true);
    /// bots.remove(&["Chrome-Lighthouse"]);
    /// assert_eq!(bots.is_bot("Chrome-Lighthouse"), false);
    ///
    /// let bot_patterns_to_remove = vec!["bingpreview/", "Google Favicon"];
    /// bots.remove(&bot_patterns_to_remove);
    /// assert_eq!(bots.is_bot("Mozilla/5.0 (Windows NT 6.1; WOW64) AppleWebKit/534+ (KHTML, like Gecko) BingPreview/1.0b"), false);
    /// assert_eq!(bots.is_bot("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/49.0.2623.75 Safari/537.36 Google Favicon"), false);
    pub fn remove(&mut self, bots: &[&str]) {
        for bot in bots {
            self.user_agent_patterns.remove(&bot.to_ascii_lowercase());
        }
        self.update_regex()
    }

    fn update_regex(&mut self) {
        self.combined_user_agent_regex = Bots::to_regex(&self.user_agent_patterns)
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
            assert_eq!(bots.is_bot(bot), true, "Invalid bot: '{}'", bot);
        }
    }

    #[test]
    fn not_bots() {
        let bots = Bots::default();
        for bot in NOT_BOTS {
            assert_eq!(bots.is_bot(bot), false, "Is a bot{}", bot);
        }
    }

    #[test]
    fn custom_user_agent_patterns() {
        let custom_user_agent_patterns = "\
            ^Simplebot\n\
            anything\\s+bot\n\
            Numerical\\d{4}\\.\\d{4}\\.\\d{4}\\.\\d{4}";
        let bots = Bots::new(custom_user_agent_patterns);
        assert_eq!(bots.is_bot("InvalidBot"), false);
        assert_eq!(bots.is_bot("Googlebot"), false);
        assert_eq!(bots.is_bot("Simplebot/1.2"), true);
        assert_eq!(bots.is_bot(" Simplebot/1.2"), false);
        assert_eq!(bots.is_bot("Anything  Bot"), true);
        assert_eq!(bots.is_bot("AnythingBot"), false);
        assert_eq!(bots.is_bot("numerical1101.2001.3987.4781"), true);
        assert_eq!(bots.is_bot("numerical1.2.3.4"), false);
    }

    #[test]
    fn empty_user_agent_patterns() {
        let empty_user_agent_patterns = "";
        let bots = Bots::new(empty_user_agent_patterns);
        assert_eq!(bots.is_bot(""), true);
        assert_eq!(bots.is_bot("1"), false);
        assert_eq!(bots.is_bot("Googlebot"), false);
    }

    #[test]
    fn single_user_agent_patterns() {
        let single_user_agent_patterns = "me";
        let bots = Bots::new(single_user_agent_patterns);
        assert_eq!(bots.is_bot(""), false);
        assert_eq!(bots.is_bot("M"), false);
        assert_eq!(bots.is_bot("Me"), true);
        assert_eq!(bots.is_bot("Googlebot"), false);
    }
    #[test]
    fn add_pattern() {
        let mut bots = Bots::default();
        assert_eq!(bots.is_bot("Mozilla/5.0 (FancyNewTestB0T /1.2)"), false);
        bots.append(&[r"FancyNewTestB0T\s/\d\.\d"]);
        assert_eq!(bots.is_bot("Mozilla/5.0 (FancyNewTestB0T /1.2)"), true);
    }

    #[test]
    fn add_multiple_patterns() {
        let mut bots = Bots::default();
        assert_eq!(bots.is_bot("Mozilla/5.0 (FancyNewTestB0T /1.2)"), false);
        assert_eq!(bots.is_bot("Special/1.0"), false);
        assert_eq!(bots.is_bot("GoogleMetaverse/2.1 (experimental)"), false);

        let new_bot_patterns = vec!["FancyNewTestB0T", "^GoogleMetaverse", "^Special/"];
        bots.append(&new_bot_patterns);

        assert_eq!(bots.is_bot("Mozilla/5.0 (FancyNewTestB0T /1.2)"), true);
        assert_eq!(bots.is_bot("Special/1.0"), true);
        assert_eq!(bots.is_bot("GoogleMetaverse/2.1 (experimental)"), true);
    }

    #[test]
    fn remove_pattern() {
        let mut bots = Bots::default();
        assert_eq!(bots.is_bot("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/78.0.3904.97 Safari/537.36 Chrome-Lighthouse"), true);
        bots.remove(&["Chrome-Lighthouse"]);
        assert_eq!(bots.is_bot("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/78.0.3904.97 Safari/537.36 Chrome-Lighthouse"), false);
        assert_eq!(bots.is_bot("Chrome-Lighthouse"), false);
        assert_eq!(bots.is_bot("Mozilla/5.0 (Windows NT 10.0; Win64; x64) adbeat.com/policy AppleWebKit/537.36 (KHTML, like Gecko) Chrome/73.0.3683.86 Safari/537.36"), true);
    }

    #[test]
    fn remove_multiple_patterns() {
        let mut bots = Bots::default();
        assert_eq!(bots.is_bot("Datadog Agent/5.10.1"), true);
        assert_eq!(bots.is_bot("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/78.0.3904.97 Safari/537.36 Chrome-Lighthouse"), true);
        assert_eq!(bots.is_bot("Mozilla/5.0 (Java) outbrain"), true);
        assert_eq!(
            bots.is_bot("Mozilla/5.0 (compatible; Google-Site-Verification/1.0)"),
            true
        );

        let bot_patterns_to_remove =
            vec!["datadog agent", "Chrome-Lighthouse", "outbrain", "google-"];
        bots.remove(&bot_patterns_to_remove);

        assert_eq!(bots.is_bot("Datadog Agent/5.10.1"), false);
        assert_eq!(bots.is_bot("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/78.0.3904.97 Safari/537.36 Chrome-Lighthouse"), false);
        assert_eq!(bots.is_bot("Mozilla/5.0 (Java) outbrain"), false);
        assert_eq!(
            bots.is_bot("Mozilla/5.0 (compatible; Google-Site-Verification/1.0)"),
            false
        );
    }
}
