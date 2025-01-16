# Use an official Rust image as the base image for building
FROM rust:alpine3.20 AS builder

# Set the working directory for the build
WORKDIR /

RUN apk add build-base musl-dev openssl-dev

# Copy the Cargo.toml and Cargo.lock files first (for dependency caching)
COPY Cargo.toml Cargo.lock ./

# Create a dummy main.rs to prebuild dependencies
RUN mkdir -p src && echo "fn main() {}" > src/main.rs

# Pre-build dependencies
RUN cargo build --release

# Now copy the full source code
COPY . .

# # Build the actual project in release mode
# RUN cargo build --release

# # Use a lightweight image for the runtime
# FROM rust:alpine3.20 AS runner

# # Install required libraries for Rust binaries
# RUN apk add --no-cache build-base musl-dev openssl-dev gcompat libssl3 libgcc libstdc++ tini ca-certificates

# # Copy the built binary from the builder stage
# COPY --from=builder /target/release/taille-auth /bin/taille-auth

# # # Make the binary executable
# RUN chmod +x /bin/taille-auth

# # Set the PORT environment variable (Railway sets this automatically)
# ENV PORT=3000

# # Expose the port
# EXPOSE ${PORT}

# # Command to run the applicastion
# ENTRYPOINT ["tini", "--"]
# CMD ["/bin/taille-auth"]