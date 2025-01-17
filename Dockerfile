# Use cargo-chef to prepare the recipe for the dependencies
FROM rust:alpine3.20 AS planner

WORKDIR /

RUN apk add build-base musl-dev openssl-dev curl

# Install cargo chef dependency.
RUN cargo install cargo-chef

COPY . .

# Now prepare the crates for release mode
RUN cargo chef prepare --recipe-path recipe.json

# Use an official Rust image as the base image for building
FROM rust:alpine3.20 AS builder

# Set the working directory for the build
WORKDIR /

RUN apk add build-base musl-dev openssl-dev curl

RUN cargo install cargo-chef

# Copy the recipe from the planner
COPY --from=planner /recipe.json recipe.json

# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json

# Pre-build dependencies
RUN cargo build --release

# Now copy the full source code
COPY . .

# Remove cached build from dummy main.rs
RUN rm -rf /target/release/taille-auth

# Build the actual project in release mode
RUN cargo build --release

# Use a lightweight image for the runtime
FROM rust:alpine3.20 AS runner

# Install required libraries for Rust binaries
RUN apk add --no-cache musl-dev openssl-dev libssl3 tini ca-certificates

# Copy the built binary from the builder stage
COPY --from=builder /target/release/taille-auth /bin/taille-auth

# # Make the binary executable
RUN chmod +x /bin/taille-auth

# Command to run the applicastion
ENTRYPOINT ["tini", "--"]
CMD ["/bin/taille-auth"]