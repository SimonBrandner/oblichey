#!/bin/sh
cd "$(dirname "$0")"
. ./models.sh

rm -rf $zipped_models
mkdir -p $zipped_models
7z -v25m a $zipped_models/models.zip $unzipped_models/*
