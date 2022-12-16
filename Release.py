#!/usr/bin/env python3

####################################################################################
# This script will generate all cpp/h files from data in the Data directory.       #
# Run this script every time data files are updated in the Squally/Data directory. #
###################################################################################$

from os import listdir
from os import path
from os.path import isfile, join, splitext, abspath, realpath, basename, relpath
import json
import os
import re
import sys

import importlib.util

def main():
    currentPath = os.path.dirname(__file__)
    
    for root, dirnames, filenames in os.walk(currentPath):
        for filename in filenames:
            if filename.lower().endswith(".csproj"):
                replaceVersionInFile(join(root, filename), "3.0.2")
                continue
	
def replaceVersionInFile(filename, newVersion):
    fin = open(filename, "r")
    
    lines = []

    for line in fin:
        line = re.sub("<Version>.+</Version>", "<Version>" + newVersion + "</Version>", line)
        line = re.sub("<AssemblyVersion>.+</AssemblyVersion>", "<AssemblyVersion>" + newVersion + "</AssemblyVersion>", line)
        line = re.sub("<FileVersion>.+</FileVersion>", "<FileVersion>" + newVersion + "</FileVersion>", line)
        lines.append(line)
       
    fin.close()
    
    fout = open(filename, "w") 
    for line in lines:
        fout.write(line)
    fout.close()
    
if __name__ == '__main__':
    main()
