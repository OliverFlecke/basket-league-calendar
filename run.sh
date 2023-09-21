#!/usr/bin/env sh

/app/geckodriver >> /dev/null &
ID=$!
sleep 1
/app/basket-calendar
kill $ID
