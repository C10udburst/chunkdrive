# Use the official Rust image as the base
FROM rust:alpine as builder

# Set the working directory inside the container
WORKDIR /app

# copy source code
COPY . .

# Install build dependencies
RUN apk add --no-cache musl-dev

# Install cargo dependencies
RUN cargo fetch

# Build the application with optimizations
RUN cargo build --release

# Use a new stage for the final image
FROM alpine:latest

# Install system dependencies
RUN apk --no-cache add ca-certificates

# Set the working directory inside the container
WORKDIR /app

# Copy the binary from the builder stage
COPY --from=builder /app/target/release/chunkdrive ./

# Copy the style.css file
COPY style.css ./

# Set the command to run the application
CMD ["./chunkdrive"]
