use chrono::{DateTime, NaiveDateTime, TimeZone};
use chrono_tz::{Europe::Copenhagen, Tz};
use derive_getters::Getters;
use thirtyfour::{prelude::ElementQueryable, By, WebElement};

#[derive(Debug, Getters)]
pub struct MatchEvent {
    home_team: String,
    away_team: String,
    time: DateTime<Tz>,
    location: String,
}

pub async fn parse_event(element: WebElement) -> anyhow::Result<Option<MatchEvent>> {
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
pub async fn extract_teams(element: &WebElement) -> anyhow::Result<Vec<String>> {
    let team_elements = element.query(By::ClassName("team-name-full")).all().await?;
    let mut teams = Vec::new();
    for team in team_elements {
        teams.push(team.text().await.unwrap());
    }

    Ok(teams)
}

/// Extract the time of the match.
/// This is always extracted in timezone +2.
pub async fn extract_time(element: &WebElement) -> anyhow::Result<DateTime<Tz>> {
    let match_time_wrapper = element.query(By::ClassName("match-time")).first().await?;
    let time_element = match_time_wrapper
        .find(By::Tag("span"))
        .await?
        .text()
        .await?;

    parse_datetime(&time_element)
}

/// Extract the location for the match.
pub async fn extract_location(element: &WebElement) -> anyhow::Result<String> {
    let location_element = element.query(By::ClassName("venuename")).first().await?;
    Ok(location_element.text().await?)
}

fn parse_datetime(value: &str) -> anyhow::Result<DateTime<Tz>> {
    let pattern = "%b %d, %Y, %I:%M %p";
    let naive_date_time = NaiveDateTime::parse_from_str(value, pattern)?;
    let local_date_time = Copenhagen.from_local_datetime(&naive_date_time).unwrap();

    Ok(local_date_time)
}

#[cfg(test)]
mod tests {
    use super::parse_datetime;
    use chrono::{DateTime, FixedOffset, NaiveDate};

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

    #[test]
    fn parse_date_in_other_timezone() {
        let s = "Nov 16, 2023, 2:30 PM";

        let expected = NaiveDate::from_ymd_opt(2023, 11, 16)
            .unwrap()
            .and_hms_opt(13, 30, 0)
            .unwrap();
        assert_eq!(
            parse_datetime(s).unwrap(),
            DateTime::<FixedOffset>::from_naive_utc_and_offset(
                expected,
                FixedOffset::east_opt(1 * 3600).unwrap()
            )
        );
    }
}
