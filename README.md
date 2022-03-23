# isbot
[![CI](https://github.com/BryanMorgan/isbot/workflows/CI/badge.svg?event=push)](https://github.com/BryanMorgan/isbot/actions)
[![Security Audit](https://github.com/BryanMorgan/isbot/actions/workflows/security-audit.yml/badge.svg)](https://github.com/BryanMorgan/isbot/actions/workflows/security-audit.yml)

<img src="./.github/logo.png" width="100" align="right">

Rust library to detect bots using a user-agent string. 

#### Features

- Focused on speed, simplicity, and ensuring real devices and browsers don't get falsely identified as bots
- Tested on over *12k* bot user agents and *180k* browser user agents - updated bot and browser lists are downloaded as part of the integration test suite
- Includes a default collection of 300+ known bot user-agent regular expressions at compile time
- Allows user-agent patterns to be manually added and removed at runtime

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
isbot = "0.1.0"
```

The example below uses the default bot patterns to correctly identify the `Googlebot-Image` user-agent as a bot and the `Opera` user-agent as a browser.

```rust
use isbot::Bots;

let bots = Bots::default();

assert_eq!(bots.is_bot("Googlebot-Image/1.0"), true);
assert_eq!(bots.is_bot("Opera/9.60 (Windows NT 6.0; U; en) Presto/2.1.1"), false);
```

## Customizing
Bot user-agent patterns can be customized by adding or removing patterns, using the `append` and `remove` methods.

### Add bot pattern
To add new bot patterns, use `append` to specify an array of regular expression patterns. For example:

```rust
let mut bots = isbot::Bots::default();

assert_eq!(bots.is_bot("Mozilla/5.0 (CustomNewTestB0T /1.2)"), false);

bots.append(&[r"CustomNewTestB0T\s/\d\.\d"]);

assert_eq!(bots.is_bot("Mozilla/5.0 (CustomNewTestB0T /1.2)"), true);
```

### Remove bots
To remove bot patterns, use `remove` and specify an array of existing patterns to remove. For example, to remove the Chrome Lighthouse user-agent pattern to indicate it is not a bot:
```rust
let mut bots = isbot::Bots::default();

bots.remove(&["Chrome-Lighthouse"]);

assert_eq!(bots.is_bot("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/78.0.3904.97 Safari/537.36 Chrome-Lighthouse"), false);
```

### Custom Bot list
The default user-agent regular expression patterns are managed in the [bot_patterns.txt](./src/bot_patterns.txt) file.

If you don't want to use the default bot patterns you can supply your own list. Since the default bot patterns are automatically added to the library at compile time you should first disable the default feature. The `include-default-bots` feature is enabled by default so the patterns defined in `bot_patterns.txt` are included in the library at compile time.

You can exclude the patterns by disabling the default features and then including your own bot regular expressions. To do that set `default-features` to false in your `Cargo.toml` dependency definition. For example:

```toml
[dependencies]
isbot = { version = "0.1.0", default-features = false }
```

And then use `Bots::new()` to supply a newline delimited list of regular expressions. For example:

```rust
use isbot::Bots;

let custom_user_agent_patterns = r#"
^Googlebot-Image/
bingpreview/"#;

let bots = Bots::new(custom_user_agent_patterns);
assert_eq!(bots.is_bot("Googlebot-Image/1.0"), true);
```

## Testing
Some of the test fixture data is download from multiple sources to ensure the latest user-agents are validated. 

To download the latest test data fixures, run the `download_fixture_data.rs` executable:

```bash
cargo run --bin download_fixture_data --features="download-fixture-data"
```

This will update files in the [fixtures](./fixtures/) directory.

### Unit and integration tests
To run all unit and integration tests:

```bash
cargo test
```

## Philosophy
Bot detection is a gray area since there are no clear lines on what defines a bot user-agent and a real device user-agent. Some libraries focus on broadly classifying bots and trying to identify as many as possible, with the risk that some real devices may be caught and falsely flagged as bots.

This library's focus is on identifying known bots while primarily ensuring no real users or devices are falsely flagged. All of the bot user-agent patterns are validated against a large number of real devices and bot patterns to ensure devices are not incorrectly identified.

For example, the user-agent string below is identified as both a bot and a real device by various libraries and data sources:

```javascript
Mozilla/5.0 (Linux; Android 4.2.1; CUBOT GT99 Build/JOP40D) AppleWebKit/535.19 (KHTML, like Gecko) Chrome/18.0.1025.166 Mobile Safari/535.19
````

- **myip.ms** -> [bot](https://myip.ms/view/web_bots/1742760/Known_Web_Bots_Mozilla_5_0_Linux_Android_4_2_1_CUBOT_GT99_Build_JOP40D_AppleWebKit_535_19_KHTML_like_Gecko_Chrome_18_0_1025_166_Mobile_Safari_535_19.html)

- **user-agents.net** -> [browser](https://user-agents.net/string/mozilla-5-0-linux-android-4-2-1-cubot-gt99-build-jop40d-applewebkit-535-19-khtml-like-gecko-chrome-18-0-1025-166-mobile-safari-535-19)

## Credits
There are many excellent bot detection libraries available for other languages and awesome developers maintaining bot and user-agent identification data. This library draws inspiration from many of them, especially:
| Library  | Language |
| ------------- | ------------- |
| https://github.com/omrilotan/isbot   | JavaScript  |
| https://github.com/JayBizzle/Crawler-Detect/  | PHP  |
| https://github.com/matomo-org/device-detector | PHP |
| https://github.com/fnando/browser | Ruby |
| https://github.com/biola/Voight-Kampff | Ruby |


The following data sources are used directly or as inspiration for the static test data and downloaded user-agent identification:
| Data Source  | Notes |
| ------------- | ------------- |
| https://user-agents.net/bots | User Agents Database |
| https://myip.ms/files/bots/live_webcrawlers.txt   | List of IP addresses of Known Web Bots & Spiders in Myip.ms Database |
| https://github.com/monperrus/crawler-user-agents   | Collection of user-agents used by robots, crawlers, and spiders  |
| https://github.com/ua-core | Regex file necessary to build language ports of Browserscope's user agent parser| 


## License
`isbot` is distributed under the terms of the MIT license. See [LICENSE](./LICENSE) for details.