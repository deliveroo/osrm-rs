#!/bin/sh

mkdir test-data
cd test-data

# Includes UAE, which has some unroutable routes.
wget -N http://download.geofabrik.de/asia/gcc-states-latest.osm.pbf

# This was confirmed to work with v5.26.0 which unfortunately did not have a dedicated tag at the
# time of writing. Any future breakage could be caused by a newer version so try modifying the tag
# from `latest` to `v5.26.0` which will hopefully be available.
docker run -t -v $(pwd):/data osrm/osrm-backend:latest osrm-extract -p /opt/foot.lua /data/gcc-states-latest.osm.pbf
docker run -t -v $(pwd):/data osrm/osrm-backend:latest osrm-contract /data/gcc-states-latest.osrm
