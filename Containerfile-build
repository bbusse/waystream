ARG ALPINE_VERSION=3.18
FROM rust:alpine${ALPINE_VERSION}
LABEL maintainer="Björn Busse <bj.rn@baerlin.eu>"
LABEL org.opencontainers.image.source https://github.com/bbusse/waystream

ARG ALPINE_VERSION

ENV USER="build-user" \
    APK_ADD="gstreamer-dev musl-dev gst-plugins-rs-dev"

RUN apk add -X https://dl-cdn.alpinelinux.org/alpine/v3.18/main -u alpine-keys --allow-untrusted

RUN echo $'http://dl-cdn.alpinelinux.org/alpine/edge/community' >> /etc/apk/repositories \
    && echo $'http://dl-cdn.alpinelinux.org/alpine/edge/testing' >> /etc/apk/repositories \
    && apk update \
    && apk upgrade \
    && apk add --no-cache $APK_ADD

RUN mkdir -p /usr/local/src/waystream
COPY . /usr/local/src/waystream
RUN cd /usr/local/src/waystream && cargo build --release

# Add application user
RUN addgroup -S $USER && adduser -S $USER -G $USER

# Add entrypoint
USER $USER
CMD ["waystream"]
