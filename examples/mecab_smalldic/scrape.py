import csv
import sys

w = csv.writer(sys.stdout, lineterminator = '\n')
for row in csv.reader(sys.stdin):
    pos = '-'.join(x for x in row[4:8] if x != '*')
    pron = row[13]
    w.writerow(row[:4] + [pos, pron if pron != '*' else ''])
