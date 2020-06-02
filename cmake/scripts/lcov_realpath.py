#!/usr/bin/env python3
import os
import sys

with open(sys.argv[1]) as f:
    for l in f:
        if l.startswith("SF:"):
            realpath = os.path.realpath(l[3:])
            print("SF:" + realpath, end='')
        else:
            print(l, end='')
