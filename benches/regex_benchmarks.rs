use criterion::{black_box, criterion_group, criterion_main, Criterion};
use isbot::Bots;
use regex::RegexSet;

const BROWSER_TEST_PATTERNS: &str = include_str!("../fixtures/browsers.txt");
const BOT_PATTERNS: &str = include_str!("../src/bot_regex_patterns.txt");

fn get_browser_user_agents() -> Vec<String> {
    BROWSER_TEST_PATTERNS
        .lines()
        .map(ToString::to_string)
        .take(100000)
        .collect::<Vec<String>>()
}

fn benchmark_browser_user_agents(c: &mut Criterion) {
    let mut group = c.benchmark_group("Browsers");
    group.sample_size(10);

    group.bench_function("Bot::is_bot", |b| {
        let bots = Bots::default();
        let browser_user_agents = get_browser_user_agents();

        b.iter(|| {
            for user_agent in &browser_user_agents {
                bots.is_bot(black_box(user_agent));
            }
        })
    });

    group.bench_function("RegexSet", |b| {
        let bot_patterns = BOT_PATTERNS
            .lines()
            .filter(|l| !l.trim().is_empty())
            .collect::<Vec<&str>>();
        let bot_patterns_regex = RegexSet::new(&bot_patterns).expect("Invalid regular expression");
        let browser_user_agents = get_browser_user_agents();

        b.iter(|| {
            for user_agent in &browser_user_agents {
                bot_patterns_regex.is_match(black_box(user_agent));
            }
        })
    });

    group.finish();
}

criterion_group!(benches, benchmark_browser_user_agents);
criterion_main!(benches);
