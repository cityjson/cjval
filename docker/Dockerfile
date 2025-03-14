
# Use the official Rust image as a parent image
FROM rust:latest as builder

USER root

# Set the working directory in the container
WORKDIR /usr/src/cjval

# Copy the current directory contents into the container
COPY . ./

# Compile the project
RUN cargo build --release --features build-binary 

# Use a minimal base image for the final image
FROM debian:bookworm-slim

ARG VERSION
LABEL org.opencontainers.image.authors="Gina Stavropoulou<g.stavropoulou@tudelft.nl>"
LABEL org.opencontainers.image.vendor="3D geoinformation group"
LABEL org.opencontainers.image.title="cjval"
LABEL org.opencontainers.image.description="Validate cityJSON files"
LABEL org.opencontainers.image.version=$VERSION
LABEL org.opencontainers.image.license="APACHE-2.0"

USER root

RUN apt-get update && apt-get install -y libssl3 && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/src/cjval/target/release/cjval /usr/local/bin/cjval

# Make the binaries available in the PATH
ENV PATH="/usr/local/bin/cjval:${PATH}"

# Define the command to run the binary by default
CMD ["cjval"]