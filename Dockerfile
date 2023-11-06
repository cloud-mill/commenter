# build stage
FROM rust:1.67 as builder

# install protobuf compiler
RUN apt-get update && apt-get install -y protobuf-compiler

WORKDIR /svc

COPY . .

RUN cargo build --release --bin commenter

# create a new stage with a minimal runtime image
FROM debian:bullseye-slim

# set the working directory to the root folder
WORKDIR /svc

RUN apt-get update \
    && apt-get install -y ca-certificates tzdata \
    && rm -rf /var/lib/apt/lists/*

# copy the built binary from the build stage
COPY --from=builder /svc/target/release/commenter .

RUN mkdir -p /etc/commenter

ENV TZ="Australia/Sydney" \
    APP_USER=catache

# run the service
CMD ["sh", "-c", "./commenter"]
