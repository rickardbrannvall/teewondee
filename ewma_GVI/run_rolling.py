#import numpy as np
#import pandas as pd
#import matplotlib.pyplot as plt

from glob import glob
import os

from subprocess import Popen, PIPE, STDOUT

allfiles = glob("data/*/*.enc")
#data = "CGM_p77_24h_2_3_400"
data = "CGMDIFF_p77_24h_2_3_50"

win = 8
for filepath in allfiles:
    _, keys, filename = filepath.split("/")
    name, _ = filename.split(".")
    temp = name.split("_")
    file = "data/%s/%s_rolling_%d.enc"%(keys,name,win)
    #flag = temp[-1]=="TIR" or temp[-1]=="ADV" or temp[-1]=="SJU"
    flag = data in name #and "6_4" in keys
    flag = flag and temp[-1]=="ABS50" or temp[-1]=="RDV"
    flag = flag and "hourly" not in name and "rolling" not in name
    #print(keys, name, len(temp), not os.path.isfile(file))
    if len(temp) >= 6 and flag and not os.path.isfile(file):
        script = "target/debug/calc_rolling %s %s %d"%(keys, name, win)
        print("", script)
        #!$script
        with Popen(script, shell=True, stdout=PIPE, stderr=STDOUT, 
                                        bufsize=1, universal_newlines=True) as p:
            for line in p.stdout:
                print(line, end='') # process line here
