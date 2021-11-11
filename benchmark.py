#!/usr/bin/env python3

import pandas
import matplotlib.pyplot as plt


def main():
    data = pandas.read_csv('./logs/benchmark.csv', header=0)
    data['granted_avg'] = data['granted_avg'] * data['count'] / 4
    grouped = data.groupby(['count', 'type']).mean()

    print(grouped)

    grouped['granted_avg'] = 1000 / grouped['granted_avg']
    fig = grouped['granted_avg'].unstack().plot(kind='bar')
    fig.set_xlabel('Number of concurrent clients')
    fig.set_ylabel('Avg response time [ms]')
    fig.set_title(
        'Response time relative to number of concurrent clients')
    fig.legend()

    # plt.semilogy()
    plt.tight_layout()
    plt.savefig('logs/response-time-granted.png')
    plt.clf()

    grouped['rejected_avg'] = 1000 / grouped['rejected_avg']
    fig = grouped['rejected_avg'].unstack().plot(kind='bar')
    fig.set_xlabel('Number of concurrent clients')
    fig.set_ylabel('Avg response time [ms]')
    fig.set_title(
        'Response time relative to number of concurrent clients')
    fig.legend()

    # plt.semilogy()
    plt.tight_layout()
    plt.savefig('logs/response-time-rejected.png')
    plt.clf()


if __name__ == "__main__":
    main()
