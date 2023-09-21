#!/usr/bin/env sh

geckodriver >> /dev/null &
ID=$!
./target/release/basket-calendar
kill $ID
