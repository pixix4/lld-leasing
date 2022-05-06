#!/usr/bin/env python3

import os
import sys
import pandas
import matplotlib.pyplot as plt


def generate(name, colors):
    csvPath = os.path.abspath("./evaluate-{}.csv".format(name))
    dir = os.path.abspath(os.path.join(csvPath, os.pardir, "evaluate"))

    data = pandas.read_csv(csvPath, header=0)
    grouped = data.groupby(['count', 'type'], sort=False).mean()

    fig = grouped['granted_avg'].unstack().plot(kind='bar', color=colors)
    fig.set_xlabel('Number of concurrent clients')
    fig.set_ylabel('Avg response time [ms]')
    fig.legend()

    plt.tight_layout()
    plt.savefig(os.path.join(dir, '{}.png'.format(name)))
    plt.clf()


def main():
    generate("phase1", ["#3498db"])
    generate("phase2", ["#3498db", "#f1c40f"])
    generate("phase3", ["#206694", "#e67e22"])
    generate("phase3-compare-naive", ["#3498db", "#206694"])
    generate("phase4", ["#e67e22", "#1abc9c"])



if __name__ == "__main__":
    main()
