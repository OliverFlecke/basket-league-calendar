use chrono::{Duration, Utc};
use icalendar::{Calendar, Class, Event, Property, *};
use std::{fs::File, io::Write};

fn main() -> anyhow::Result<()> {
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
