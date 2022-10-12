#!/usr/bin/env bash
# This script is meant to be run on Unix/Linux based systems

set -e


# Check the current directory to ensure the user
# executes the script successfully.
if [ ! -d "scripts" ]; then
  echo "You need to run this script from the project root folder."
  exit 1
fi


# Load variables from env vars file, if currently defined
if [ -f ".env" ]; then
  export $(cat .env | xargs)
fi


# Check if the environment file is correctly defined
if [ ! -f ".env" ] || \
  [[ -z "${COMPOSE_PROJECT_NAME}" ]] || \
  [[ -z "${DOMAIN}" ]] || \
  [[ -z "${EMAIL}" ]]; then
  echo "Complete the following information to deploy the node:"

  # Get two parameters to associate with the
  # RPC Let's Encrypt certificate.
  #
  # - Domain name
  # - Domain email address
  #
  read -p "[RPC Domain Name]: " DOMAIN
  read -p "[RPC Domain Email Address]: " EMAIL

  # Export the parameters values to the project environment file
  echo COMPOSE_PROJECT_NAME="all-in-app" > .env
  echo DOMAIN=${DOMAIN} >> .env
  echo EMAIL=${EMAIL} >> .env
fi


echo "*** Starting All in Network node ***"


# Remove old infrastructure
docker compose down -v --remove-orphans

# Prepare RPC Let's Encrypt certificate
docker compose -f ./docker-compose-initiate.yml up -d nginx
docker compose -f ./docker-compose-initiate.yml up certbot
docker compose -f ./docker-compose-initiate.yml down

# some configurations for let's encrypt
curl -L --create-dirs -o conf/rpc/letsencrypt/options-ssl-nginx.conf https://raw.githubusercontent.com/certbot/certbot/master/certbot-nginx/certbot_nginx/_internal/tls_configs/options-ssl-nginx.conf

openssl dhparam -out conf/rpc/letsencrypt/ssl-dhparams.pem 2048

# Define a cron job to renew the Let's Encrypt certificate
crontab ./conf/rpc/crontab/crontab.conf

# Deploy the new infrastructure
docker compose -f ./docker-compose.yml up -d
