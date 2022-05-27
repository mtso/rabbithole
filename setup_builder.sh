#!/bin/sh
set -e

curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source $HOME/.cargo/env

yum groupinstall -y "Development Tools"
dnf install -y pkg-config openssl-devel
