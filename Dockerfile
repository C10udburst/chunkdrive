FROM alpine:latest

# Install system dependencies
RUN apk --no-cache add ca-certificates

# Set the working directory inside the container
WORKDIR /app

# Copy the binary
COPY --chown=root:root ./target/*/release/chunkdrive ./

# Copy the style.css file
COPY --chown=root:root style.css ./

# Set permissions
RUN chmod 755 chunkdrive
RUN chmod 644 style.css

USER root

# Set the command to run the application
CMD ["/bin/sh", "-c", "/app/chunkdrive"]
