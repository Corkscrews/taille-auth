# Use an official Rust image as the base image for building
FROM rust:latest AS builder

# Set the working directory for the build
WORKDIR /usr/src/app

# Copy the entire project into the container
COPY . .

# Build the Rust project in release mode
RUN cargo build --release

# Use a lightweight image for the runtime
FROM ubuntu:latest

# Install required libraries for Rust binaries
RUN apt-get update && apt-get install -y \
    libssl-dev \
    ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# Set the working directory for the runtime
WORKDIR /app

# Copy the built binary from the builder stage
COPY --from=builder /usr/src/app/target/release/taille-auth /app/taille-auth

# Make the binary executable
RUN chmod +x /app/taille-auth

# Expose the port for Railway (use the $PORT environment variable)
ENV PORT=3000
EXPOSE 3000

# Command to run the application
CMD ["./taille-auth", "--port", "$PORT"]