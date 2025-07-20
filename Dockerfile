# Build stage
FROM rust:1.75-alpine AS builder

# Install build dependencies
RUN apk add --no-cache musl-dev pkgconfig openssl-dev

# Create app directory
WORKDIR /usr/src/lazycelery

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src

# Build the application
RUN cargo build --release --target x86_64-unknown-linux-musl

# Runtime stage
FROM alpine:3.19

# Install runtime dependencies
RUN apk add --no-cache ca-certificates

# Copy the binary from builder
COPY --from=builder /usr/src/lazycelery/target/x86_64-unknown-linux-musl/release/lazycelery /usr/local/bin/lazycelery

# Create non-root user
RUN addgroup -g 1000 lazycelery && \
    adduser -D -u 1000 -G lazycelery lazycelery

# Switch to non-root user
USER lazycelery

# Set the entrypoint
ENTRYPOINT ["lazycelery"]

# Default command (can be overridden)
CMD ["--help"]