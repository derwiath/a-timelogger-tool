use chrono::{Datelike, Duration, NaiveDateTime, NaiveTime};
use std::cmp;
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

fn read_report<'a>(file_contents: &'a str) -> Vec<ReportEntry<'a>> {
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

fn day_reports_from_entries(entries: &Vec<ReportEntry>, year: i32) -> Vec<DayReport> {
    let mut day_reports = Vec::<DayReport>::new();
    for entry in entries {
        let from_dt = parse_datetime(year, entry.from).unwrap();
        let to_dt = parse_datetime(year, entry.to).unwrap();
        if to_dt >= from_dt {
            let mut dt = from_dt.clone();
            while dt <= to_dt {
                let date = dt.date();
                let pos = if let Some(pos) = day_reports.iter().position(|x| x.date == date) {
                    pos
                } else {
                    day_reports.push(DayReport::new(date));
                    day_reports.len() - 1
                };
                let report = &mut day_reports.get_mut(pos).unwrap();

                let next_date = date.checked_add_signed(Duration::days(1)).unwrap();
                let midnight = NaiveDateTime::new(next_date, NaiveTime::from_hms(0, 0, 0));
                let duration_to_midnight = midnight.signed_duration_since(dt);
                let duration_to_dt = to_dt.signed_duration_since(dt);
                let seconds = cmp::min(duration_to_midnight, duration_to_dt).num_seconds();
                report.seconds += seconds;
                dt = midnight;
            }
        } else {
            panic!("{} < {}", to_dt, from_dt);
        }
    }
    day_reports.sort_by(|a, b| a.date.cmp(&b.date));
    day_reports
}

fn process_file(filename: &str, print_remainder: bool) {
    let file_contents = fs::read_to_string(filename).expect("Failed to read report file");
    let entries = read_report(&file_contents);
    let year = chrono::Local::now().year();

    let day_reports = day_reports_from_entries(&entries, year);

    let mut month_output: Option<u32> = None;
    let mut week_output: Option<u32> = None;

    for report in day_reports.iter() {
        let month = report.date.month();
        let new_month = match month_output.replace(month) {
            Some(old_month) => month != old_month,
            None => true,
        };
        if new_month {
            let month_day_reports = day_reports.iter().filter_map(|report| {
                if report.date.month() == month {
                    Some(report)
                } else {
                    None
                }
            });
            let minutes: i64 = month_day_reports
                .clone()
                .filter_map(|report| Some(report.seconds / 60))
                .filter_map(|minutes| Some((minutes / 3) * 3))
                .sum();
            let duration = chrono::Duration::minutes(minutes);
            let duration_remainder = if print_remainder {
                let minutes_remainder: i64 = month_day_reports
                    .filter_map(|report| Some(report.seconds / 60))
                    .filter_map(|minutes| Some(minutes % 3))
                    .sum();
                Some(chrono::Duration::minutes(minutes_remainder))
            } else {
                None
            };
            print!(
                "{} {}   {:03}:{:02}",
                month_to_string(month),
                report.date.year(),
                duration.num_hours(),
                duration.num_minutes() % 60,
            );
            if let Some(remainder) = duration_remainder {
                print!(" +{}", remainder.num_minutes());
            }
            println!();
            week_output = None
        }
        let week = report.date.iso_week().week();
        let new_week = match week_output.replace(week) {
            Some(old_week) => week != old_week,
            None => true,
        };
        if new_week {
            let week_day_reports = day_reports.iter().filter_map(|report| {
                if report.date.month() == month && report.date.iso_week().week() == week {
                    Some(report)
                } else {
                    None
                }
            });
            let minutes: i64 = week_day_reports
                .clone()
                .filter_map(|report| Some(report.seconds / 60))
                .filter_map(|minutes| Some((minutes / 3) * 3))
                .sum();
            let duration = chrono::Duration::minutes(minutes);
            let duration_remainder = if print_remainder {
                let minutes_remainder: i64 = week_day_reports
                    .filter_map(|report| Some(report.seconds / 60))
                    .filter_map(|minutes| Some(minutes % 3))
                    .sum();
                Some(chrono::Duration::minutes(minutes_remainder))
            } else {
                None
            };
            print!(
                "  Week {:02}   {:02}:{:02}",
                week,
                duration.num_hours(),
                duration.num_minutes() % 60,
            );
            if let Some(remainder) = duration_remainder {
                print!(" +{}", remainder.num_minutes());
            }
            println!();
        }

        let minutes = report.seconds / (60 * 3) * 3;
        let duration = chrono::Duration::minutes(minutes);
        let duration_remainder = if print_remainder {
            let minutes_remainder = report.seconds / 60 % 3;
            Some(chrono::Duration::minutes(minutes_remainder))
        } else {
            None
        };
        print!(
            "    {:02} {}  {:02}:{:02}",
            report.date.day(),
            report.date.weekday(),
            duration.num_hours(),
            duration.num_minutes() % 60,
        );
        if let Some(remainder) = duration_remainder {
            print!(" +{}", remainder.num_minutes());
        }
        println!();
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let name = args.get(0).expect("Failed to get executable name");
    let filename = args
        .get(1)
        .expect(format!("Usage: {} <report-csv>", name).as_str());
    process_file(&filename, true);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_entry_spanning_two_midnights_yield_three_day_reports() {
        let year = 2020;
        let day1_str = "1 Jan";
        let day2_str = "2 Jan";
        let day3_str = "3 Jan";
        let line = format!("Work;0,2;{} 23:59;{} 00:02;", day1_str, day3_str);
        let entries = read_report(&line);
        let day_reports = day_reports_from_entries(&entries, year);

        let date1 = chrono::NaiveDate::parse_from_str(
            format!("{} {}", year, day1_str).as_str(),
            "%Y %d %b",
        )
        .unwrap();
        assert_eq!(day_reports[0].date, date1);
        assert_eq!(day_reports[0].seconds, 60);

        let date2 = chrono::NaiveDate::parse_from_str(
            format!("{} {}", year, day2_str).as_str(),
            "%Y %d %b",
        )
        .unwrap();
        assert_eq!(day_reports[1].date, date2);
        assert_eq!(day_reports[1].seconds, 24 * 60 * 60);

        let date3 = chrono::NaiveDate::parse_from_str(
            format!("{} {}", year, day3_str).as_str(),
            "%Y %d %b",
        )
        .unwrap();
        assert_eq!(day_reports[2].date, date3);
        assert_eq!(day_reports[2].seconds, 120);

        assert_eq!(day_reports.len(), 3);
    }
}
