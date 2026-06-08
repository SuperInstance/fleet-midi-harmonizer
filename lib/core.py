"""Core operations for this fleet MIDI service."""
def process(v, base=60):
    n = [base]
    for x in v:
        if x == 1: n.append(n[-1]+4)
        elif x == -1: n.append(n[-1]-4)
        else: n.append(n[-1])
    return n
