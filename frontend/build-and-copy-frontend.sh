#!/usr/bin/env bash

lein clean &&
lein cljsbuild once min &&
cp -rf resources/public .