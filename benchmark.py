#!/usr/bin/env python3

import os
import sys
import pandas
import matplotlib.pyplot as plt


def main():
    csvPath = os.path.abspath(sys.argv[1])
    dir = os.path.abspath(os.path.join(csvPath, os.pardir))

    data = pandas.read_csv(csvPath, header=0)
    grouped = data.groupby(['count', 'type'], sort=False).mean()

    print(grouped)
    f = open(os.path.join(dir, 'benchmark-groubed.txt'), "w")
    f.write(str(grouped))
    f.close()

    fig, ax = plt.subplots(1, 2, figsize=(10, 5))
    fig = grouped['granted_avg'].unstack().plot(kind='bar', ax=ax[0])
    fig.set_xlabel('Number of concurrent clients')
    fig.set_ylabel('Avg response time [ms]')
    fig.legend()

    fig = grouped['granted_avg'].unstack().plot(kind='bar', ax=ax[1])
    fig.set_xlabel('Number of concurrent clients ($90^{th}$ percentile)')
    fig.set_ylabel('Avg response time [ms]')
    fig.legend()
    fig.set_ylim(0, grouped['granted_avg'].quantile(0.9))

    # plt.semilogy()
    plt.tight_layout()
    plt.savefig(os.path.join(dir, 'response-time-granted.png'))
    plt.clf()

    fig, ax = plt.subplots(1, 2, figsize=(10, 5))
    fig = grouped['rejected_avg'].unstack().plot(kind='bar', ax=ax[0])
    fig.set_xlabel('Number of concurrent clients')
    fig.set_ylabel('Avg response time [ms]')
    fig.legend()

    fig = grouped['rejected_avg'].unstack().plot(kind='bar', ax=ax[1])
    fig.set_xlabel('Number of concurrent clients ($90^{th}$ percentile)')
    fig.set_ylabel('Avg response time [ms]')
    fig.legend()
    fig.set_ylim(0, grouped['rejected_avg'].quantile(0.9))

    # plt.semilogy()
    plt.tight_layout()
    plt.savefig(os.path.join(dir, 'response-time-rejected.png'))
    plt.clf()

    fig = grouped['timeout_count'].unstack().plot(kind='bar')
    fig.set_xlabel('Number of concurrent clients')
    fig.set_ylabel('Number of timeouts')
    fig.legend()

    # plt.semilogy()
    plt.tight_layout()
    plt.savefig(os.path.join(dir, 'response-count-timeout.png'))
    plt.clf()


if __name__ == "__main__":
    main()
