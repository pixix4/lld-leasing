#!/usr/bin/env python3

import pandas
import matplotlib.pyplot as plt


def main():
    data = pandas.read_csv('./logs/benchmark.csv', header=0)
    grouped = data.groupby(['count', 'type']).mean()

    print(grouped)

    fig = grouped['average'].unstack().plot(kind='bar')
    fig.set_xlabel('Number of concurrent connections')
    fig.set_ylabel('Response time [ms]')
    fig.set_title(
        'Response times relative to number of concurrent connections')
    fig.legend()

    plt.tight_layout()
    plt.savefig('logs/response-time.png')
    plt.clf()

    fig = grouped['errors'].unstack().plot(kind='bar')
    fig.set_xlabel('Number of concurrent connections')
    fig.set_ylabel('Number of network errors')
    fig.set_title(
        'Network errors relative to number of concurrent connections')
    fig.legend()

    plt.tight_layout()
    plt.savefig('logs/network-errors.png')
    plt.clf()


if __name__ == "__main__":
    main()
