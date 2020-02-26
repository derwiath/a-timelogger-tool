use chrono::{Datelike, NaiveDateTime};
use itertools::Itertools;
use std::collections::HashMap;
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
    year: i32,
    entry_datetime: &str,
) -> Result<chrono::NaiveDateTime, chrono::ParseError> {
    let s = format!("{} {}", year, entry_datetime);
    NaiveDateTime::parse_from_str(s.as_str(), "%Y %d %b %H:%M")
}

fn month_to_string(month: u32) -> &'static str {
    match month {
        1 => "Jan",
        2 => "Feb",
        3 => "Mar",
        4 => "Apr",
        5 => "May",
        6 => "Jun",
        7 => "Jul",
        8 => "Aug",
        9 => "Sep",
        10 => "Oct",
        11 => "Nov",
        12 => "Dec",
        _ => panic!("{} is not a valid month", month),
    }
}

struct DayReport {
    date: chrono::NaiveDate,
    seconds: i64,
}

impl DayReport {
    fn new(date: chrono::NaiveDate) -> DayReport {
        DayReport { date, seconds: 0 }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let name = args.get(0).expect("Failed to get executable name");
    let filename = args
        .get(1)
        .expect(format!("Usage: {} <report-csv>", name).as_str());
    let file_contents = fs::read_to_string(filename).expect("Failed to read report file");
    let entries = read_report(&file_contents);
    let year = chrono::Local::now().year();

    let mut day_reports = HashMap::new();
    for entry in entries {
        let from_dt = parse_datetime(year, entry.from).unwrap();
        let to_dt = parse_datetime(year, entry.to).unwrap();
        assert_eq!(from_dt.date(), to_dt.date());
        let date = from_dt.date();
        let duration = to_dt.signed_duration_since(from_dt);
        let report = day_reports.entry(date).or_insert(DayReport::new(date));
        report.seconds += duration.num_seconds();
    }

    let mut month_output: Option<u32> = None;
    let mut week_output: Option<u32> = None;

    for date in day_reports.keys().sorted() {
        let month = date.month();
        if match month_output.replace(month) {
            Some(old_month) => month != old_month,
            None => true,
        } {
            let seconds: i64 = day_reports
                .values()
                .filter_map(|report| {
                    if report.date.month() == month {
                        Some(report.seconds)
                    } else {
                        None
                    }
                })
                .sum();
            let duration = chrono::Duration::seconds(seconds);
            println!(
                "{} {}   {:03}:{:02}",
                month_to_string(month),
                date.year(),
                duration.num_hours(),
                duration.num_minutes() % 60
            );
            week_output = None
        }
        let week = date.iso_week().week();
        if match week_output.replace(week) {
            Some(old_week) => week != old_week,
            None => true,
        } {
            let seconds: i64 = day_reports
                .values()
                .filter_map(|report| {
                    if report.date.month() == month && report.date.iso_week().week() == week {
                        Some(report.seconds)
                    } else {
                        None
                    }
                })
                .sum();
            let duration = chrono::Duration::seconds(seconds);
            println!(
                "  Week {:02}   {:02}:{:02}",
                week,
                duration.num_hours(),
                duration.num_minutes() % 60
            );
        }

        let report = day_reports.get(date).unwrap();
        let duration = chrono::Duration::seconds(report.seconds);
        println!(
            "    {:02} {}  {:02}:{:02}",
            date.day(),
            date.weekday(),
            duration.num_hours(),
            duration.num_minutes() % 60
        );
    }
}
