#!/usr/bin/env bash
# This script is meant to be run on Unix/Linux based systems
set -e

echo "*** Starting All in Network node ***"

# Evaluate current directory
# ...


# Takes three parameters to associate with the certificate:
#
# - Application name
# - Domain name
# - Email
#

COMPOSE_PROJECT_NAME=$1
DOMAIN=$2
EMAIL=$3


# Description
echo COMPOSE_PROJECT_NAME=${COMPOSE_PROJECT_NAME} > .env
echo DOMAIN=${DOMAIN} >> .env
echo EMAIL=${EMAIL} >> .env


# Phase 1
docker compose -f ./docker-compose-initiate.yml up -d nginx
docker compose -f ./docker-compose-initiate.yml up certbot
docker compose -f ./docker-compose-initiate.yml down

# some configurations for let's encrypt
curl -L --create-dirs -o conf/rpc/letsencrypt/options-ssl-nginx.conf https://raw.githubusercontent.com/certbot/certbot/master/certbot-nginx/certbot_nginx/_internal/tls_configs/options-ssl-nginx.conf

openssl dhparam -out conf/rpc/letsencrypt/ssl-dhparams.pem 2048

# Phase 2
crontab ./conf/rpc/crontab/crontab.conf
docker compose -f ./docker-compose.yml up -d
