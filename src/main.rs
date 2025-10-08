use clap::{Arg, Command};
use gittree::app::App;
use gittree::config::Config;
use gittree::git::{FilterOptions, Repository};
use std::process;

fn main() {
    let matches = Command::new("gittree")
        .version("0.1.0")
        .about("A fast TUI that renders an ASCII/Unicode commit tree like GitHub's network graph")
        .arg(
            Arg::new("unicode")
                .long("unicode")
                .help("Use Unicode lane characters")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("no-color")
                .long("no-color")
                .help("Disable colors")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("since")
                .long("since")
                .help("Show commits more recent than a specific date")
                .value_name("DATE"),
        )
        .arg(
            Arg::new("until")
                .long("until")
                .help("Show commits older than a specific date")
                .value_name("DATE"),
        )
        .arg(
            Arg::new("author")
                .short('a')
                .long("author")
                .help("Limit commits to author (regex)")
                .value_name("PATTERN"),
        )
        .arg(
            Arg::new("path")
                .long("path")
                .help("Limit commits to path (repeatable)")
                .value_name("PATH"),
        )
        .arg(
            Arg::new("range")
                .long("range")
                .help("Rev range (e.g. main..feature)")
                .value_name("RANGE"),
        )
        .arg(
            Arg::new("max-commits")
                .long("max-commits")
                .help("Cap log read (0 = no limit)")
                .value_name("N")
                .value_parser(clap::value_parser!(usize)),
        )
        .arg(
            Arg::new("pager")
                .long("pager")
                .help("Use $PAGER for details")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("yes")
                .long("yes")
                .help("Skip confirmations")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("backend")
                .long("backend")
                .help("Backend to use (go, rs)")
                .value_name("BACKEND")
                .default_value("rs"),
        )
        .arg(
            Arg::new("style")
                .long("style")
                .help("Style (light, dark, auto)")
                .value_name("STYLE")
                .default_value("auto"),
        )
        .arg(
            Arg::new("version")
                .long("version")
                .help("Show version information")
                .action(clap::ArgAction::SetTrue),
        )
        .get_matches();

    if matches.get_flag("version") {
        println!("gittree 0.1.0 (Rust backend)");
        process::exit(0);
    }

    // Load configuration
    let config = Config::load().unwrap_or_else(|_| Config::default());

    // Parse date filters
    let since = matches.get_one::<String>("since").map(|s| parse_time(s));
    let until = matches.get_one::<String>("until").map(|s| parse_time(s));

    // Create git repository instance
    let repo = match Repository::new(".") {
        Ok(repo) => repo,
        Err(e) => {
            eprintln!("Failed to open git repository: {}", e);
            process::exit(1);
        }
    };

    // Create filter options
    let filter = FilterOptions {
        author: matches.get_one::<String>("author").cloned(),
        path: matches.get_one::<String>("path").cloned(),
        since,
        until,
        range: matches.get_one::<String>("range").cloned(),
        max_commits: matches.get_one::<usize>("max-commits").copied(),
    };

    // Create and run the app
    let mut app = App::new(repo, config, filter);
    if let Err(e) = app.run() {
        eprintln!("Application error: {}", e);
        process::exit(1);
    }
}

fn parse_time(s: &str) -> chrono::DateTime<chrono::Utc> {
    // Try common formats
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(s) {
        return dt.with_timezone(&chrono::Utc);
    }
    
    if let Ok(naive_dt) = chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S") {
        return chrono::DateTime::from_naive_utc_and_offset(naive_dt, chrono::Utc);
    }
    
    if let Ok(naive_dt) = chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d") {
        return chrono::DateTime::from_naive_utc_and_offset(
            naive_dt.and_hms_opt(0, 0, 0).unwrap(),
            chrono::Utc,
        );
    }

    // Handle relative time formats
    let now = chrono::Utc::now();
    match s {
        "2w" => now - chrono::Duration::weeks(2),
        "1w" => now - chrono::Duration::weeks(1),
        "3d" => now - chrono::Duration::days(3),
        "2d" => now - chrono::Duration::days(2),
        "1d" => now - chrono::Duration::days(1),
        "12h" => now - chrono::Duration::hours(12),
        "6h" => now - chrono::Duration::hours(6),
        "1h" => now - chrono::Duration::hours(1),
        _ => {
            eprintln!("Unable to parse time: {}", s);
            process::exit(1);
        }
    }
}