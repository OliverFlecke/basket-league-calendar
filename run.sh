#!/usr/bin/env sh

/app/geckodriver >/dev/null 2>&1 &
ID=$!
sleep 1
/app/basket-calendar
kill $ID
