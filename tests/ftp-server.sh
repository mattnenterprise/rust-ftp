#!/bin/sh
docker run -ti --privileged -p 21:21 -p 65000-65010:65000-65010 ftp-server
