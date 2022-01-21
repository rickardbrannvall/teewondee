#import numpy as np
#import pandas as pd
#import matplotlib.pyplot as plt

from glob import glob
import os

from subprocess import Popen, PIPE, STDOUT


allfiles = glob("data/*/*.enc")
data = "CGM_p77_24h_2_3_400"
#data = "CGMDIFF_p77_24h_2_3_50"


for filepath in allfiles:
    _, keys, filename = filepath.split("/")
    name, _ = filename.split(".")
    temp = name.split("_")
    file = "data/%s/%s_CGM_hourly.enc"%(keys,name)
    #print(keys, name, len(temp), os.path.isfile(file))
    flag = data in name #and "6_4" in keys
    #flag = flag and temp[-1]=="ABS50" or temp[-1]=="RDV"
    flag = flag and "hourly" not in name and "rolling" not in name
    if len(temp) >= 6 and flag and not os.path.isfile(file):
        script = "target/debug/calc_hourly %s %s"%(keys, name)
        print("", script)
        #!$script
        with Popen(script, shell=True, stdout=PIPE, stderr=STDOUT, 
                                        bufsize=1, universal_newlines=True) as p:
            for line in p.stdout:
                print(line, end='') # process line here
