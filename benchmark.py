#!/usr/bin/env python3

import pandas
import matplotlib.pyplot as plt


def main():
    data = pandas.read_csv('./logs/benchmark.csv', header=0)
    grouped = data.groupby(['count', 'type']).mean()

    print(grouped)

    fig = grouped['granted_avg'].unstack().plot(kind='bar')
    fig.set_xlabel('Number of concurrent clients')
    fig.set_ylabel('Avg response time [ms]')
    fig.legend()

    plt.semilogy()
    plt.tight_layout()
    plt.savefig('logs/response-time-granted.png')
    plt.clf()

    fig = grouped['rejected_avg'].unstack().plot(kind='bar')
    fig.set_xlabel('Number of concurrent clients')
    fig.set_ylabel('Avg response time [ms]')
    fig.legend()

    plt.semilogy()
    plt.tight_layout()
    plt.savefig('logs/response-time-rejected.png')
    plt.clf()

    fig = grouped['timeout_count'].unstack().plot(kind='bar')
    fig.set_xlabel('Number of concurrent clients')
    fig.set_ylabel('Number of timeouts')
    fig.legend()

    plt.semilogy()
    plt.tight_layout()
    plt.savefig('logs/response-count-timeout.png')
    plt.clf()


if __name__ == "__main__":
    main()
