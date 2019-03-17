#!/bin/bash

for filename in 01/*.tst; do
	echo "Testing ${filename}"
	HardwareSimulator.sh $filename
done
