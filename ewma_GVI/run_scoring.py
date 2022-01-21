#import numpy as np
#import pandas as pd
#import matplotlib.pyplot as plt

from glob import glob
import os

from subprocess import Popen, PIPE, STDOUT

#data, idx = "CGMDIFF_p77_24h_2_3_50", "ABS50"
#data, idx = "CGM_p77_24h_2_3_400_SJU", "ADV50"
data, idx = "CGMDIFF_p77_24h_2_3_50", "RDV"
allfiles = glob("data/*/%s*.enc"%data)

for filepath in allfiles:
  #for idx in ["ADV50"]: #"TIR","GVP","LOG", "SIX", "SJU", "ADV"
    _, keys, filename = filepath.split("/")
    name, _ = filename.split(".")
    tmp = name.split("_")
    file = filepath.split(".")[0]+"_%s.enc"%idx
    if len(tmp) in [6,7] and not os.path.isfile(file) and name==data:
        script = "target/debug/calc_scores %s %s %s"%(keys, name, idx)
        print(script)
        #!$script       
        with Popen(script, shell=True, stdout=PIPE, stderr=STDOUT, 
                                        bufsize=1, universal_newlines=True) as p:
            for line in p.stdout:
                print(line, end='') # process line here

 