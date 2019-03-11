#!/bin/sh

mkdir test-data
cd test-data

# Includes UAE, which has some unroutable routes.
wget -N http://download.geofabrik.de/asia/gcc-states-latest.osm.pbf

docker run -t -v $(pwd):/data osrm/osrm-backend:v5.21.0 osrm-extract -p /opt/foot.lua /data/gcc-states-latest.osm.pbf
docker run -t -v $(pwd):/data osrm/osrm-backend:v5.21.0 osrm-contract /data/gcc-states-latest.osrm
