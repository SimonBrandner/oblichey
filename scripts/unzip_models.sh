#!/bin/sh
cd "$(dirname "$0")"
. ./models.sh

rm -rf $unzipped_models
mkdir -p $unzipped_models
7z x $zipped_models/models.zip.001 -o$unzipped_models
