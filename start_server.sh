#! /bin/bash
if [ "$(id -u)" == "0" ]; then
   echo "do not start as root!" 2&>1
   exit 1
fi

sudo cp /etc/letsencrypt/live/spotitube.if-lab.de/fullchain.pem chain.pem
sudo cp /etc/letsencrypt/live/spotitube.if-lab.de/privkey.pem key.pem

sudo pkill -f chromi_tube_backend
sudo $(which cargo) run --release 0.0.0.0:443
