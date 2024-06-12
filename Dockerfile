# Stage 1: Build the application
FROM rust:latest

# Install dependencies
RUN apt-get update && apt-get install -y \
    libopencv-dev \
    pkg-config \
    cmake \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*
    
# Set the working directory
WORKDIR /usr/src/app



# Copy the Cargo.toml and Cargo.lock files
COPY Cargo.toml ./

# Copy .env file
COPY .env ./.env
# Copy the source code
COPY src ./src

# Build the Rust application
RUN cargo install --path .

CMD ["image2nord"]
