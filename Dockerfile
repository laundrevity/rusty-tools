# Use an official Rust image as a base image
FROM rust:latest

# Set the working directory in the container to `/usr/src/myapp`
WORKDIR /usr/src/myapp

# Copy the current directory contents into the container at `/usr/src/myapp`
COPY . .

# Compile the current project (dependencies will be cached unless changed)
RUN cargo build

# Use Tini as a minimal init system for containers - it will act as a process subreaper for jailing processes
ENV RUST_LOG=info

# A dummy command that keeps the container running
CMD ["sleep", "infinity"]
