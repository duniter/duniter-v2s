# Duniter debug node
#
# Requires to run from repository root and to copy the binary in the build folder
# (part of the CI workflow)

FROM docker.io/library/ubuntu:20.04 AS builder

RUN apt-get update && apt-get install -y ca-certificates && update-ca-certificates

FROM debian:buster-slim
LABEL maintainer "elois@duniter.org"
LABEL description="Binary for duniter debug node"

RUN useradd -m -u 1000 -U -s /bin/sh -d /duniter duniter && \
	mkdir -p /duniter/.local/share && \
	mkdir /data && \
	chown -R duniter:duniter /data && \
	ln -s /data /duniter/.local/share/duniter && \
	rm -rf /usr/bin /usr/sbin

COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt

USER duniter

COPY --chown=duniter build/duniter /duniter/duniter
RUN chmod uog+x /duniter/duniter

# 30333 for p2p
# 9933 for RPC call
# 9944 for Websocket
# 9615 for Prometheus (metrics)
EXPOSE 30333 9933 9944 9615

VOLUME ["/data"]

ENTRYPOINT ["/duniter/duniter"]
