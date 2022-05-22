#!/bin/sh

curl -L --output reproxy_v0.11.0_linux_x86_64.tar.gz https://github.com/umputun/reproxy/releases/download/v0.11.0/reproxy_v0.11.0_linux_x86_64.tar.gz
tar -xf reproxy_v0.11.0_linux_x86_64.tar.gz
mv reproxy /usr/sbin/

mkdir -p /etc/reproxy
cp config.yml /etc/reproxy/config.yml

cp reproxy.service /etc/systemd/system/

