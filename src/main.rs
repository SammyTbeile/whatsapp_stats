use chrono::{DateTime, FixedOffset, NaiveDateTime, Datelike};
use clap::{Parser, ValueEnum};
use regex::Regex;
use std::collections::HashMap;
use std::fs;

#[derive(Parser,Debug)]
#[command(version, about, long_about= None)]
struct Args {
    /// The input file
    path: String,

    /// Print out per year stats
    #[arg(short, long, action)]
    year: bool,

    /// Print out per user stats
    #[arg(short, long, action)]
    user: bool,

    /// Pretty print the table
    #[arg(short, long, action)]
    pretty: bool,

    /// Sort by messages or words
    #[arg(long, value_enum, default_value_t = SortBy::Messages)]
    sort: SortBy,

}

#[derive(Debug, Clone)]
struct Message {
    timestamp: chrono::DateTime<FixedOffset>,
    author: String,
    text: String,
}


#[derive(Debug)]
struct Stat {
    user: String, // the user
    num_messages: u64, // the number of messages the user sent
    num_words: u64, // the number of words the user sent
    first_message: DateTime<FixedOffset>, // the date of the first message the user sent
    percent_messages: f32, // the percentage of all messages that the user sent
    percent_words: f32 // the percentage of all words that the user sent
}

#[derive(ValueEnum, Clone, Debug)]
enum SortBy {
    Messages,
    Words
}

fn parse(content: &str) -> Vec<Message> {
    // Parses the file to a vec of messages
    let re = Regex::new(r"^\[(\d{1,2}/\d{1,2}/\d{2}), (\d{1,2}:\d{2}:\d{2}\u{202F}[AP]M)] (.*?): (.*)").unwrap();
    let tz_offset = FixedOffset::west_opt(0).unwrap();

    let mut messages = Vec::new();
    let mut buffer = String::new();
    let mut current_timestamp = None;
    let mut current_author = String::new();

    for line in content.lines() {
        if let Some(caps) = re.captures(line) {
            if let Some(timestamp) = current_timestamp.take() {
                messages.push(Message {
                    timestamp,
                    author: current_author.clone(),
                    text: buffer.trim().to_string(),
                });
            }

            let datetime_str = format!("{} {}", &caps[1], &caps[2].replace('\u{202F}', " "));
            let naive = NaiveDateTime::parse_from_str(&datetime_str, "%m/%d/%y %I:%M:%S %p").unwrap();
            current_timestamp = Some(DateTime::from_naive_utc_and_offset(naive, tz_offset));
            current_author = caps[3].to_string();
            buffer = caps[4].to_string();
        } else {
            buffer.push('\n');
            buffer.push_str(line);
        }
    }

    if let Some(timestamp) = current_timestamp {
        messages.push(Message {
            timestamp,
            author: current_author,
            text: buffer.trim().to_string(),
        });
    }

    messages
}


fn compute_stats(messages: &[Message]) -> Vec<Stat> {
    // Given a vec of messages, calculate:
    // 1. the number of messages each user sent
    // 2. the number of words each user sent
    // Optionally, only calculate the statistics for a given year and/or given user
    // Return a vec of Stat
        let mut map: HashMap<String, Stat> = HashMap::new();
    let total_messages = messages.len() as f32;
    let total_words: f32 = messages.iter().map(|m| m.text.split_whitespace().count() as f32).sum();

    for m in messages {
        let entry = map.entry(m.author.clone()).or_insert_with(|| Stat {
            user: m.author.clone(),
            num_messages: 0,
            num_words: 0,
            first_message: m.timestamp,
            percent_messages: 0.0,
            percent_words: 0.0,
        });
        entry.num_messages += 1;
        entry.num_words += m.text.split_whitespace().count() as u64;
        if m.timestamp < entry.first_message {
            entry.first_message = m.timestamp;
        }
    }

    for stat in map.values_mut() {
        stat.percent_messages = stat.num_messages as f32 / total_messages * 100.0;
        stat.percent_words = stat.num_words as f32 / total_words * 100.0;
    }

    map.into_values().collect()
}

fn print_stats(mut stats: Vec<Stat>, pretty: bool, sort: SortBy) {
    // A function that builds and pretty prints a table of the format:
    // User | num_messages | num_words | percent_messages | percent_words | first_message
    // The table should sort the list by either messages or words based on sorting
    match sort {
        SortBy::Messages => stats.sort_by_key(|s| std::cmp::Reverse(s.num_messages)),
        SortBy::Words => stats.sort_by_key(|s| std::cmp::Reverse(s.num_words)),
    }

    if pretty {
        use tabled::{Table, Tabled};

        #[derive(Tabled)]
        struct DisplayStat {
            User: String,
            Messages: u64,
            Words: u64,
            First: String,
            percent_messages: String,
            percent_words: String,
        }

        let display: Vec<DisplayStat> = stats
            .into_iter()
            .map(|s| DisplayStat {
                User: s.user,
                Messages: s.num_messages,
                Words: s.num_words,
                First: s.first_message.format("%Y-%m-%d %H:%M:%S").to_string(),
                percent_messages: format!("{:.2}%", s.percent_messages),
                percent_words: format!("{:.2}%", s.percent_words),
            })
            .collect();

        let table = Table::new(display);
        println!("{}", table);
    } else {
        for s in stats {
            println!(
                "{}: {} msgs, {} words, first at {}, {:.2}% msgs, {:.2}% words",
                s.user,
                s.num_messages,
                s.num_words,
                s.first_message,
                s.percent_messages,
                s.percent_words
            );
        }
    }
}



fn main() {
    let args = Args::parse();
    let content = fs::read_to_string(&args.path).expect("Failed to read file");
    let messages = parse(&content);

    if args.user {
        if args.year {
            let mut grouped: HashMap<i32, Vec<Message>> = HashMap::new();
            for msg in &messages {
                grouped.entry(msg.timestamp.year()).or_default().push(msg.clone());
            }

            for (year, msgs) in grouped.into_iter().collect::<std::collections::BTreeMap<_,_>>() {
                println!("\n=== Stats for {} ===", year);
                let stats = compute_stats(&msgs);
                print_stats(stats, args.pretty, args.sort.clone());
            }
        } else {
            let stats = compute_stats(&messages);
            print_stats(stats, args.pretty, args.sort);
        }
    }
}
