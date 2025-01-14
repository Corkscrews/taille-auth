# Use an Ubuntu image for the final runtime
FROM ubuntu:latest

# Install required libraries for Rust binaries
RUN apt-get update && apt-get install -y \
    libssl-dev \
    ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# Set the working directory
WORKDIR /app

# Copy the pre-built binary into the image
COPY ./target/release/taille-auth /app/taille-auth

# Expose the port for Railway (use the $PORT environment variable)
ENV PORT=3000
EXPOSE 3000

# Command to run the application
CMD ["sh", "-c", "./taille-auth --port $PORT"]
