FROM alpine:latest

# Install system dependencies
RUN apk --no-cache add ca-certificates

# Set the working directory inside the container
WORKDIR /app

# Copy the binary
COPY /app/target/*/release/chunkdrive ./

# Copy the style.css file
COPY style.css ./

# Set the command to run the application
CMD ["./chunkdrive"]
