FROM alpine:latest

# Install system dependencies
RUN apk --no-cache add ca-certificates

# Create a group and user
RUN addgroup -S chunkdrive && adduser -S chunkdrive -G chunkdrive
USER chunkdrive:chunkdrive

# Set the working directory inside the container
WORKDIR /app

# Copy the binary
COPY ./target/*/release/chunkdrive ./

# Copy the style.css file
COPY style.css ./

# Set the command to run the application
CMD ["/app/chunkdrive"]
