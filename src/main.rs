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
    #[arg(short, long, default_value = "Basket")]
    calendar: String,
    #[arg(short, long, default_value = "BK Amager")]
    team: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Cli::parse();
    let matches = get_all_events_from_page(&args.team).await?;
    create_calendar_of_matches(&args.calendar, matches)?;

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
        let start = CalendarDateTime::from((event.time().naive_utc(), event.time().timezone()));
        let end = CalendarDateTime::from((
            event.time().naive_utc() + chrono::Duration::hours(2),
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

fn create_calendar_of_matches(calendar_name: &str, matches: Vec<MatchEvent>) -> anyhow::Result<()> {
    let calendar = matches.iter().fold(Calendar::new(), |mut cal, event| {
        cal.push(Event::from(event));
        cal
    });

    let filename = format!("{}.ics", calendar_name);
    println!("Saving calendar to file: {filename}");

    let mut output = File::create(filename)?;
    write!(output, "{calendar}")?;

    Ok(())
}
