use chrono::NaiveDateTime;
use std::env;
use std::fs;

extern crate chrono;

#[derive(Debug)]
struct ReportEntry<'a> {
    activity: &'a str,
    duration: &'a str,
    from: &'a str,
    to: &'a str,
    comment: &'a str,
}

impl<'a> ReportEntry<'_> {
    pub fn new(
        activity: &'a str,
        duration: &'a str,
        from: &'a str,
        to: &'a str,
        comment: &'a str,
    ) -> ReportEntry<'a> {
        ReportEntry {
            activity,
            duration,
            from,
            to,
            comment,
        }
    }
}

fn read_report<'a>(file_contents: &'a String) -> Vec<ReportEntry<'a>> {
    let mut entries = Vec::<ReportEntry<'a>>::new();
    for line in file_contents.lines() {
        let tokens: Vec<&str> = line.split(';').collect();
        if tokens.len() != 5 {
            break;
        }
        if tokens[0] == "Activity type" {
            continue;
        }
        entries.push(ReportEntry::new(
            tokens[0], tokens[1], tokens[2], tokens[3], tokens[4],
        ));
    }
    entries
}

fn parse_datetime(
    year: u32,
    entry_datetime: &str,
) -> Result<chrono::NaiveDateTime, chrono::ParseError> {
    let s = format!("{} {}", year, entry_datetime);
    NaiveDateTime::parse_from_str(s.as_str(), "%Y %d %b %H:%M")
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let name = args.get(0).expect("Failed to get executable name");
    let filename = args
        .get(1)
        .expect(format!("Usage: {} <report-csv>", name).as_str());
    let file_contents = fs::read_to_string(filename).expect("Failed to read report file");
    let entries = read_report(&file_contents);
    let year: u32 = 2020;

    for entry in entries {
        println!("{:?}", entry);
        let from_dt = parse_datetime(year, entry.from).unwrap();
        let to_dt = parse_datetime(year, entry.to).unwrap();
        assert_eq!(from_dt.date(), to_dt.date());
        let date = from_dt.date();
        let duration = to_dt.signed_duration_since(from_dt);
        println!(
            "  duration {:02}:{:02}",
            duration.num_hours(),
            duration.num_minutes() % 60
        );
    }
}
