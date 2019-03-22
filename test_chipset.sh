#!/bin/bash

for filename in {01,02}/*.tst; do
	echo "Testing ${filename}"
	HardwareSimulator.sh $filename
done

for filename in $(find 03 -name *.tst); do
	echo "Testing ${filename}"
	HardwareSimulator.sh $filename
done
