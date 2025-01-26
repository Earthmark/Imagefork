FROM rust AS builder
WORKDIR /usr/local/src/imagefork
COPY . .
RUN cargo install --path .

FROM alpine
#COPY --from=builder \
#  /usr/local/src/imagefork/templates \
#  /usr/local/etc/imagefork/templates
#COPY --from=builder \
#  /usr/local/src/imagefork/www \
#  /usr/local/etc/imagefork/www
#COPY --from=builder \
#  /usr/local/src/imagefork/images \
 # /usr/local/etc/imagefork/images
#RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/imagefork /usr/local/bin/imagefork
WORKDIR /usr/local/etc/imagefork
CMD ["imagefork"]