####################################################################################################
## Builder
####################################################################################################
FROM rust:latest AS builder

RUN rustup target add x86_64-unknown-linux-musl
RUN apt update && apt install -y musl-tools musl-dev
RUN update-ca-certificates

# Create appuser
ENV USER=app
ENV UID=10001

RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    "${USER}"

WORKDIR /app

# Copy the Cargo.toml and Cargo.lock files first to leverage Docker's caching mechanism
COPY Cargo.toml Cargo.lock ./

# Fetch the dependencies
RUN cargo build --target x86_64-unknown-linux-musl --release

# Copy the source code into the container
COPY ./src ./src

# Build the application
RUN cargo build --target x86_64-unknown-linux-musl --release

####################################################################################################
## Final image
####################################################################################################
FROM scratch

# Import from builder.
COPY --from=builder /etc/passwd /etc/passwd
COPY --from=builder /etc/group /etc/group

WORKDIR /app

# Copy the compiled binary from the builder stage
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/url_shortener_api ./

# Use an unprivileged user.
USER app:app

CMD ["/app/url_shortener_api"]
