# Default Argos Dockerfile

# This file can be used as a template for your own projects.
# Copy this to ArgosCI/Dockerfile in your project to get started.

FROM rust:latest

# Install clippy and rustfmt
RUN rustup component add clippy rustfmt

# Install any extra system dependencies here
# RUN apt-get update && apt-get install -y libssl-dev pkg-config

# Set up a working directory
WORKDIR /app

# The actual command (cargo ...) is passed via docker run
# Argos handles mounting and permissions automatically.
