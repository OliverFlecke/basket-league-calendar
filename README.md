# Basket calendar extractor

Small utility tool for extracting the calendar of games in the Danish basket league. 
Used to get the calendar events for my brother's matches in the league.

## Requirements 

- Rust (tested with rustc v. 1.72.0)
- Firefox
- [geckodriver](https://github.com/mozilla/geckodriver)


## Run 

First start `geckodriver` and then run the script:

```sh
geckodriver &
cargo run
```

This will create a new calendar file (`.ics`) in the current directory that can be imported.
