def save(var, file):
    import pickle
    with open(file, 'wb') as f:
        pickle.dump(var, f)

def load(file):
    import pickle
    with open(file, 'rb') as f:
        return pickle.load(f)

def groupby(it, key=lambda x: x, value=lambda x: x):
    result = {}
    for item in it:
        k = key(item)
        if k in result:
            result[k].append(value(item))
        else:
            result[k] = [value(item)]
    return result

def car(x):
    return x[0]

def cdr(x):
    return x[1:]

def cadr(x):
    return x[1]

def normalize(v):
    norm = sum(v)
    if norm == 0:
       raise Exception("normalizae all 0")
    return [x / norm for x in v]
