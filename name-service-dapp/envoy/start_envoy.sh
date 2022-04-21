#!/usr/bin/env bash

# now we can start envoy in a docker container and map the configuration and service definition inside
# we use --network="host" so that envoy can access the grpc service at localhost:<port>
# the envoy-config.yml has configured envoy to run at port 8080, grpc-web can connect to 8080


if ! [ -x "$(command -v docker)" ] ; then
    echo "docker command is not available, please install docker"
    echo "Install docker: https://store.docker.com/search?offering=community&type=edition"
    exit 1
fi

# check if sudo is required to run docker
if [ "$(groups | grep -c docker)" -gt "0" ]; then
    echo "Envoy will run at port 8080 (see envoy.yaml)"
    docker run -it --rm --name envoy  \
             -p 8080:8080 -p 9901:9901 \
             -v "$(pwd)/envoy.yaml:/etc/envoy/envoy.yaml:ro" \
             envoyproxy/envoy:v1.17.0
else
    echo "you are not in the docker group, running with sudo"
    echo "Envoy will run at port 8080 (see envoy.yaml)"
    sudo docker run -it --rm --name envoy \
             -p 8080:8080 -p 9901:9901 \
             -v "$(pwd)/envoy.yaml:/etc/envoy/envoy.yaml:ro" \
             envoyproxy/envoy:v1.17.0
fi