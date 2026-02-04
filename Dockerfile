# Frontend build stage
FROM node:20-slim AS frontend-builder

WORKDIR /app/stark-frontend

COPY stark-frontend/package*.json ./
RUN npm ci
COPY stark-frontend/ ./
RUN npm run build

# Backend build stage
FROM rust:1.88-slim-bookworm AS backend-builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

# Copy only what's needed for the build
COPY Cargo.toml Cargo.lock ./
COPY stark-backend ./stark-backend

# Build the application
RUN cargo build --release -p stark-backend

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    sqlite3 \
    && rm -rf /var/lib/apt/lists/*

# Copy the binary
COPY --from=backend-builder /app/target/release/stark-backend /app/

# Copy the built frontend
COPY --from=frontend-builder /app/stark-frontend/dist /app/stark-frontend/dist

# Copy config, ABIs, skills, and SOUL.md
COPY config /app/config
COPY abis /app/abis
COPY skills /app/skills
COPY SOUL.md /app/SOUL.md

# Create directories
RUN mkdir -p /app/workspace /app/journal /app/soul

EXPOSE 8080

CMD ["/app/stark-backend"]
