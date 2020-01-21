#! /bin/bash
if [ "$(id -u)" == "0" ]; then
   echo "do not start as root!" 2&>1
   exit 1
fi

sudo pkill -f chromi_tube_backend
sudo ROCKET_ENV=production $(which cargo) run --release
