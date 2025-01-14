# Use the official Rust image as the base image
FROM rust:latest AS builder

# Set the working directory inside the container
WORKDIR /usr/src/app

# Copy the Cargo.toml and Cargo.lock files first to optimize caching
COPY Cargo.toml Cargo.lock ./

# Create a dummy main.rs to pre-fetch and compile dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Pre-fetch and compile dependencies
RUN cargo build --release && rm -rf src

# Copy the rest of the application source code
COPY . ./

# Build the application in release mode
RUN cargo build --release

# Use a smaller base image for the final image
FROM debian:bullseye-slim

# Install required libraries
RUN apt-get update && apt-get install -y libssl-dev ca-certificates && rm -rf /var/lib/apt/lists/*

# Set the working directory in the final image
WORKDIR /app

# Copy the compiled binary from the builder stage
COPY --from=builder /usr/src/app/target/release/taille-auth /app/taille-auth

# Expose the port for Railway (use the $PORT environment variable)
ENV PORT=3000
EXPOSE 3000

# Command to run the application
CMD ["sh", "-c", "./taille-auth --port $PORT"]
