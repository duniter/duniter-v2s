FROM docker.io/library/ubuntu:20.04

# metadata
ARG VCS_REF
ARG BUILD_DATE

# show backtraces
ENV RUST_BACKTRACE 1

# install tools and dependencies
RUN apt-get update && \
	DEBIAN_FRONTEND=noninteractive apt-get upgrade -y && \
	DEBIAN_FRONTEND=noninteractive apt-get install -y \
		libssl1.1 \
		ca-certificates \
		curl && \
# apt cleanup
	apt-get autoremove -y && \
	apt-get clean && \
	find /var/lib/apt/lists/ -type f -not -name lock -delete; \
# add user
	useradd -m -u 1000 -U -s /bin/sh -d /duniter duniter

# add duniter binary to docker image
COPY ./build/duniter /usr/local/bin

USER duniter

# check if executable works in this container
RUN /usr/local/bin/duniter --version

EXPOSE 30333 9944
VOLUME ["/duniter"]

ENTRYPOINT ["/usr/local/bin/duniter"]
