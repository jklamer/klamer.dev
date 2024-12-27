FROM rust:1.83 as builder
WORKDIR /usr/src
COPY . .
RUN cargo install --path ./klamer_dev

FROM public.ecr.aws/amazonlinux/amazonlinux:latest

RUN yum update -y && yum install -y openssl-devel
COPY --from=builder /usr/local/cargo/bin/klamer_dev /usr/local/bin/klamer_dev

EXPOSE 3000
EXPOSE 443
EXPOSE 80

CMD klamer_dev -d klamer.dev
