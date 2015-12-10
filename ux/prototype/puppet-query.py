#!/usr/bin/env python
# -*- coding: utf-8 -*-

import os
import sys
import cli_lib
import datetime
import json
import time
import yaml


def process_cli_arguments(cli, arguments):
    """Processes arguments passed to the CLI."""

    # because lazy prototype
    try:
        if arguments['--docopt']:
            cli_lib.print_cli_arguments(arguments)
        else:
            render_result(
                cli,
                arguments=arguments,
                render_as=arguments['--format'],
                query_result='default_query.json'
                )

    except KeyboardInterrupt:
        cli_lib.exit_cli()


def render_result(cli, arguments, query_result='default_query.json', render_as='tab'):
    """Spit out things that look vaguely like a query query_result"""

    if arguments['--file']:
        query_result = arguments['--file']

    if arguments['--verbose'] or arguments['--debug']:
        # Timestamp format: 2015-12-09 04:15:56.844661
        print datetime.datetime.now().strftime("%Y-%m-%d %H:%M:%S.%f") + ' INFO  puppetlabs.puppet-access.command - ',
        cli_lib.print_color('Reading configuration from /home/centos/.puppetlabs/client-tools/TOOL_NAME.conf\n', 'green'),

    if arguments['--debug']:
        for i in range(len(cli['msgs'])):
            print datetime.datetime.now().strftime("%Y-%m-%d %H:%M:%S.%f"),
            print 'DEBUG',
            print cli['msgs'][i]['command'],
            print ' - ',
            cli_lib.print_color(cli['msgs'][i]['message']+'\n', 'cyan')
            time.sleep(0.01)

    with open(query_result) as data_file:
        data = json.load(data_file)
        if render_as == 'csv':
            # TODO: Make this actually reformat something
            print cli['csv']
        elif render_as == 'json':
            print json.dumps(data)
        elif render_as == 'pretty-json':
            print json.dumps(data, indent=4)
        elif render_as == 'yaml':
            print yaml.safe_dump(data, default_flow_style=False)
        else:
            # The table representation is problematic.
            from tabulate import tabulate
            print tabulate(data, headers="keys", tablefmt="grid")

if __name__ == '__main__':
    command = os.path.splitext(os.path.basename(__file__))[0]
    cli = cli_lib.get_cli_from_command(command)
    process_cli_arguments(cli['cli'],
                          cli['arguments'])
