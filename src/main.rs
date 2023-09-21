mod extractor;

use clap::Parser;
use extractor::{parse_event, MatchEvent};
use icalendar::{Calendar, Event, *};
use std::{fs::File, io::Write, time::Duration};
use thirtyfour::prelude::*;

const URL: &str = "https://basketligaen.dk/kampprogram";
const GECKODRIVER_HOST: &str = "http://localhost:4444";

#[derive(Parser)]
#[command(author, version, about)]
struct Cli {
    #[arg(short, long, default_value = "basket")]
    calendar: String,
    #[arg(short, long, default_value = "BK Amager")]
    team: String,
    #[arg(short, long)]
    save: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Cli::parse();
    let matches = get_all_events_from_page(&args.team).await?;
    let calendar = create_calendar_of_matches(matches);

    if args.save {
        write_to_file(&calendar, &args.calendar)?;
    } else {
        calendar.print()?;
    }

    Ok(())
}

async fn get_all_events_from_page(team: &str) -> anyhow::Result<Vec<MatchEvent>> {
    let mut caps = DesiredCapabilities::firefox();
    caps.set_headless()?;
    let driver = WebDriver::new(GECKODRIVER_HOST, caps).await?;

    driver.goto(URL).await?;

    let elements = driver
        .query(By::ClassName("match-wrap"))
        .wait(Duration::from_secs(5), Duration::from_millis(100))
        .all()
        .await?;

    let mut matches = Vec::new();
    for element in elements {
        if let Some(game) = parse_event(element, team).await? {
            matches.push(game);
        }
    }

    driver.quit().await?;

    Ok(matches)
}

impl From<&MatchEvent> for Event {
    fn from(event: &MatchEvent) -> Self {
        let start = CalendarDateTime::from((event.time().naive_local(), event.time().timezone()));
        let end = CalendarDateTime::from((
            event.time().naive_local() + chrono::Duration::hours(2),
            event.time().timezone(),
        ));

        Event::new()
            .summary(&format!(
                "Basket: {} vs {}",
                event.home_team(),
                event.away_team()
            ))
            .location(event.location())
            .starts(start)
            .ends(end)
            .done()
    }
}

fn create_calendar_of_matches(matches: Vec<MatchEvent>) -> Calendar {
    let mut calendar = Calendar::new();
    calendar
        .name("BK Amager")
        .description("Game schedule for BK Amager basket team")
        .timezone("Europe/Copenhagen")
        .ttl(&chrono::Duration::hours(1));

    matches.iter().map(Event::from).for_each(|e| {
        calendar.push(e);
    });

    calendar
}

fn write_to_file(calendar: &Calendar, calendar_name: &str) -> anyhow::Result<()> {
    let filename = format!("{}.ics", calendar_name);
    println!("Saving calendar to file: {filename}");

    let mut output = File::create(filename)?;
    write!(output, "{calendar}")?;

    Ok(())
}
