FROM rust:latest

# Set the working directory inside the container
WORKDIR /usr/src/app

# Copy the Cargo.toml and Cargo.lock files first to optimize caching
COPY Cargo.toml Cargo.lock ./

# Create a dummy main.rs to build dependencies only
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Pre-fetch and compile dependencies
RUN cargo build --release && rm -rf src

# Copy the rest of the application source code
COPY . ./

# Build the application in release mode
RUN cargo build --release

# Expose the server port
EXPOSE 3000

# Command to run the application
CMD ["./target/release/taille-auth"]
