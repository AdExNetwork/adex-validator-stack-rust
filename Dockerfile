FROM rust:1.40 as builder

MAINTAINER dev@adex.network

WORKDIR /usr/src/app

COPY . .

RUN cargo install --path validator_worker --all-features

WORKDIR /usr/local/bin

RUN cp $CARGO_HOME/bin/validator_worker .

FROM ubuntu:18.04

# `ethereum` or `dummy`
ENV ADAPTER=

# only applicable if you use the `--adapter ethereum`
ENV KEYSTORE_FILE=
ENV KEYSTORE_PWD=

# Only applicable if you use the `--adapter dummy`
ENV DUMMY_IDENTITY=

# If set it will override the configuration file used
ENV CONFIG=
# Defaults to `http://127.0.0.1:8005`
ENV SENTRY_URL=
# Set to any value to run the `validator_worker` in `single tick` mode
# default: `infinite`
ENV SINGLE_TICK=

WORKDIR /usr/local/bin

RUN apt update && apt-get install -y libssl-dev ca-certificates

COPY docs/config/cloudflare_origin.crt /usr/local/share/ca-certificates/

RUN update-ca-certificates

COPY --from=builder /usr/local/bin/validator_worker .

CMD validator_worker -a ${ADAPTER:-ethereum} \
            ${KEYSTORE_FILE:+-k $KEYSTORE_FILE} \
            ${DUMMY_IDENTITY:+-i $DUMMY_IDENTITY} \
            ${SINGLE_TICK:+-t} \
            ${SENTRY_URL:+-u $SENTRY_URL} ${CONFIG}
