import json
import os, errno

def loadfile(filename):
    with open(filename, 'r', encoding='utf-8') as f:
        data = json.load(f)
    return data

def savefile(filename, data):
    if not os.path.exists(os.path.dirname(filename)):
        try:
            os.makedirs(os.path.dirname(filename))
        except OSError as exc: # Guard against race condition
            if exc.errno != errno.EEXIST:
                raise
    with open(filename , 'w', encoding='utf-8') as f:
        json.dump(data, f)