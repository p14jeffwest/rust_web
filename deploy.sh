#!/bin/bash

# Deploy scripts in the Linux(Ubuntu) 
# This script build rust code and copy all required files to `web_server` directory.

# Set deployment directory
DEPLOY_DIR="/home/ubuntu/rust_web/web_erver/"

# 1. stop web server to avoid file conflict to copy files to deploy directory
./stop_service.sh

# 2. compile the Rust code
cargo build --release

# 3. Copy necessary files to the deployment directory
rm -r "$DEPLOY_DIR"
mkdir -p "$DEPLOY_DIR"
cp -r ./target/release/rust_web "$DEPLOY_DIR"
cp -r ./log4rs.yml "$DEPLOY_DIR"
cp -r ./index.html "$DEPLOY_DIR"
cp -r ./css "$DEPLOY_DIR"
cp -r ./js "$DEPLOY_DIR"

echo "Deployment completed successfully."

# 4. remove src directory
# sudo rm -r ./src

# 4. start web server
./start_service.sh

# rust_web.service file is located in the 'etc/systemd/system' directory
# if this service file is updated, please run the following command to reload the systemd daemon
# sudo systemctl daemon-reload
# sudo systemctl enable rust_web.service 
# sudo systemctl start rust_web.service
# sudo systemctl status rust_web.service


