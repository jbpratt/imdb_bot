#!/usr/bin/env bash

files=(
	name.basics.tsv.gz
	title.akas.tsv.gz
	title.basics.tsv.gz
	title.crew.tsv.gz
	title.episode.tsv.gz
	title.principals.tsv.gz
	title.ratings.tsv.gz
)

mkdir -p data
cd data/ || exit

for file in "${files[@]}"; do
	wget --verbose "https://datasets.imdbws.com/${file}"
	gunzip "${file}"
done
