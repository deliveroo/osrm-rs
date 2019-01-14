#!/bin/sh

mkdir test-data
cd test-data

wget -N http://download.geofabrik.de/europe/germany/berlin-latest.osm.pbf

docker run -t -v $(pwd):/data osrm/osrm-backend:v5.21.0 osrm-extract -p /opt/car.lua /data/berlin-latest.osm.pbf
docker run -t -v $(pwd):/data osrm/osrm-backend:v5.21.0 osrm-contract /data/berlin-latest.osrm
