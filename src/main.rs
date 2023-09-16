use chrono::{DateTime, FixedOffset, NaiveDateTime};
use chrono_tz::Europe::Copenhagen;
use icalendar::{Calendar, Class, Event, Property, *};
use std::{fs::File, io::Write, time::Duration};
use thirtyfour::prelude::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    load_page().await?;
    Ok(())
}

const URL: &str = "https://basketligaen.dk/kampprogram";

async fn load_page() -> anyhow::Result<()> {
    let mut caps = DesiredCapabilities::firefox();
    caps.set_headless()?;
    // caps.set_no_sandbox()?;
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

    for game in matches {
        println!("{:#?}", game);
    }

    // sleep(std::time::Duration::from_secs(10)).await;

    driver.quit().await?;

    Ok(())
}

#[derive(Debug)]
struct MatchEvent {
    home_team: String,
    away_team: String,
    time: DateTime<FixedOffset>,
    location: String,
}

async fn parse_event(element: WebElement) -> anyhow::Result<Option<MatchEvent>> {
    let teams = extract_teams(&element).await?;
    if !teams.iter().any(|x| x == "BK Amager") {
        return Ok(None);
    }

    let time = extract_time(&element).await?;
    let location = extract_location(&element).await?;

    let game = MatchEvent {
        home_team: teams[0].to_owned(),
        away_team: teams[1].to_owned(),
        time,
        location,
    };

    Ok(Some(game))
}

/// Extract the teams playing.
async fn extract_teams(element: &WebElement) -> anyhow::Result<Vec<String>> {
    let team_elements = element.query(By::ClassName("team-name-full")).all().await?;
    let mut teams = Vec::new();
    for team in team_elements {
        teams.push(team.text().await.unwrap());
    }

    Ok(teams)
}

/// Extract the time of the match.
/// This is always extracted in timezone +2.
async fn extract_time(element: &WebElement) -> anyhow::Result<DateTime<FixedOffset>> {
    let match_time_wrapper = element.query(By::ClassName("match-time")).first().await?;
    let time_element = match_time_wrapper
        .find(By::Tag("span"))
        .await?
        .text()
        .await?;

    parse_datetime(&time_element)
}

/// Extract the location for the match.
async fn extract_location(element: &WebElement) -> anyhow::Result<String> {
    let location_element = element.query(By::ClassName("venuename")).first().await?;
    Ok(location_element.text().await?)
}

fn create_calendar() -> anyhow::Result<()> {
    use chrono::{Duration, Utc};
    let event = Event::new()
        .summary("test event")
        .description("here I have something really important to do")
        .starts(Utc::now())
        .class(Class::Confidential)
        .ends(Utc::now() + Duration::days(1))
        .append_property(
            Property::new("TEST", "FOOBAR")
                .add_parameter("IMPORTANCE", "very")
                .add_parameter("DUE", "tomorrow")
                .done(),
        )
        .done();

    let bday = Event::new()
        .all_day(Utc::now().date_naive())
        .summary("My Birthday")
        .description(
            r#"Hey, I'm gonna have a party
BYOB: Bring your own beer.
Hendrik"#,
        )
        .done();

    let mut calendar = Calendar::new();
    calendar.push(event);
    calendar.push(bday);

    let mut output = File::create("test.ics")?;
    write!(output, "{}", calendar)?;

    Ok(())
}

fn parse_datetime(value: &str) -> anyhow::Result<DateTime<FixedOffset>> {
    let pattern = "%b %d, %Y, %I:%M %p";

    Ok(NaiveDateTime::parse_from_str(value, pattern)?
        .and_local_timezone(FixedOffset::east_opt(2 * 3600).unwrap())
        .single()
        .unwrap())
}

#[cfg(test)]
mod tests {
    use chrono::{DateTime, FixedOffset, NaiveDate};

    use crate::parse_datetime;

    #[test]
    fn parse_date() {
        let s = "Sep 16, 2023, 2:30 PM";

        let expected = NaiveDate::from_ymd_opt(2023, 9, 16)
            .unwrap()
            .and_hms_opt(12, 30, 0)
            .unwrap();
        assert_eq!(
            parse_datetime(s).unwrap(),
            DateTime::<FixedOffset>::from_naive_utc_and_offset(
                expected,
                FixedOffset::east_opt(2 * 3600).unwrap()
            )
        );
    }
}
