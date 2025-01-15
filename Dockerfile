# Use an official Rust image as the base image for building
FROM rust:alpine3.20 AS compiler

# Set the working directory for the build
WORKDIR /

RUN apk add musl-dev

# Copy the Cargo.toml and Cargo.lock files first (for dependency caching)
COPY Cargo.toml Cargo.lock ./

# Create a dummy main.rs to prebuild dependencies
RUN mkdir -p src && echo "fn main() {}" > src/main.rs

# Pre-build dependencies
RUN cargo build --release

# Now copy the full source code
COPY . .

# Build the actual project in release mode
RUN cargo build --release

# Use a lightweight image for the runtime
FROM alpine:3.20

# Install required libraries for Rust binaries
RUN apk add --no-cache \
    libssl3 \
    tini \
    ca-certificates

# Copy the built binary from the builder stage
COPY --from=compiler /target/release/taille-auth /bin/taille-auth

# Make the binary executable
RUN chmod +x /bin/taille-auth

# Expose the port for Railway (use the $PORT environment variable)
EXPOSE 3000/tcp

# Command to run the application
ENTRYPOINT ["tini", "--"]
CMD ["/bin/taille-auth"]