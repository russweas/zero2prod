#!/usr/bin/env bash
docker run -d -p6831:6831/udp -p6832:6832/udp  -e JAEGER_DISABLED=true -p16686:16686 jaegertracing/all-in-one:latest
