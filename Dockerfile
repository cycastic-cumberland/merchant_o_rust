FROM rust:latest AS builder

RUN update-ca-certificates

# Create appuser
ENV USER=rest_server
ENV UID=10001

RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/rest_server" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    "${USER}"


WORKDIR /rest_server

COPY ./ .

RUN cargo build --release
RUN strip -s /rest_server/target/release/merchant_o_rust

FROM gcr.io/distroless/cc as final

# Import from builder.
COPY --from=builder /etc/passwd /etc/passwd
COPY --from=builder /etc/group /etc/group

WORKDIR /rest_server

# Copy our build
COPY --from=builder /rest_server/target/release/merchant_o_rust ./

# Use an unprivileged user.
USER rest_server:rest_server

CMD ["/rest_server/merchant_o_rust"]
