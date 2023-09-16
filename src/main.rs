mod extractor;

use extractor::{parse_event, MatchEvent};
use icalendar::{Calendar, Event, *};
use std::{fs::File, io::Write, time::Duration};
use thirtyfour::prelude::*;

const URL: &str = "https://basketligaen.dk/kampprogram";
const CALENDAR_FILENAME: &str = "basket.ics";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let matches = get_all_events_from_page().await?;
    create_calendar_of_matches(matches)?;

    Ok(())
}

async fn get_all_events_from_page() -> anyhow::Result<Vec<MatchEvent>> {
    let mut caps = DesiredCapabilities::firefox();
    caps.set_headless()?;
    let driver = WebDriver::new("http://localhost:4444", caps).await?;

    driver.goto(URL).await?;

    let elements = driver
        .query(By::ClassName("match-wrap"))
        .wait(Duration::from_secs(5), Duration::from_millis(100))
        .all()
        .await?;

    let mut matches = Vec::new();
    for element in elements {
        if let Some(game) = parse_event(element).await? {
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

fn create_calendar_of_matches(matches: Vec<MatchEvent>) -> anyhow::Result<()> {
    let calendar = matches.iter().fold(Calendar::new(), |mut cal, event| {
        cal.push(Event::from(event));
        cal
    });

    let mut output = File::create(CALENDAR_FILENAME)?;
    write!(output, "{calendar}")?;

    Ok(())
}
