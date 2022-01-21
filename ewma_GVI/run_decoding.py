#import numpy as np
#import pandas as pd
#import matplotlib.pyplot as plt

from glob import glob
import os

from subprocess import Popen, PIPE, STDOUT

allfiles = glob("data/*/*.enc")

for filepath in allfiles:
    _, keys, filename = filepath.split("/")
    name, _ = filename.split(".")
    file = "data/%s/%s.csv"%(keys,name)
    #print(keys, name, os.path.isfile(file))
    if not os.path.isfile(file):
        script = "target/debug/decode_data %s %s"%(keys, name)
        print("**********\n", script)
        #!$script
        with Popen(script, shell=True, stdout=PIPE, stderr=STDOUT, 
                                        bufsize=1, universal_newlines=True) as p:
            for line in p.stdout:
                print(line, end='') # process line here
 