# Use an Alpine image for the final runtime
FROM alpine:latest

# Install required libraries for Rust binaries
RUN apk add --no-cache libssl1.1 ca-certificates

# Set the working directory in the final image
WORKDIR /app

# Copy the pre-built binary into the image
COPY ./target/release/taille-auth /app/taille-auth

# Expose the port for Railway (use the $PORT environment variable)
ENV PORT=3000
EXPOSE 3000

# Command to run the application
CMD ["sh", "-c", "./taille-auth --port $PORT"]