#!/usr/bin/env python
# -*- coding: utf-8 -*-

import os
import sys
import cli_lib
import datetime
import random
from itertools import islice


def process_cli_arguments(cli, arguments):
    """Processes arguments passed to the CLI."""

    # because lazy prototype
    try:
        if arguments['--docopt']:
            cli_lib.print_cli_arguments(arguments)
        if arguments['status']:
            cli_status(cli)
        if arguments['export']:
            if arguments['<file>']:
                cli_export(cli, arguments['<file>'])
            else:
                cli_export(cli)

    except KeyboardInterrupt:
        cli_lib.exit_cli()


def cli_status(cli):
    print"""{
    "detail_level": "info",
     "service_status_version": 1,
     "service_version": "4.0.0-SNAPSHOT",
     "state": "running",
     "status": {
         "maintenance_mode?": false,
         "read_db_up?": true,
         "write_db_up?": true,
         "queue_depth": 0
     }
}"""


def cli_export(cli, outfile='puppet-db.tar.gz'):
    cli_lib.show_progress(13,"Triggering export to '"+cli_lib.add_color(outfile,'blue')+"' at " + datetime.datetime.now().strftime("%Y-%m-%d %H:%M:%S")+" ",0)
    cli_lib.show_progress(13,"Finised export to '"+cli_lib.add_color(outfile,'blue')+"' at " + datetime.datetime.now().strftime("%Y-%m-%d %H:%M:%S")+" ",0)

if __name__ == '__main__':
    command = os.path.splitext(os.path.basename(__file__))[0]
    cli = cli_lib.get_cli_from_command(command)
    process_cli_arguments(cli['cli'],
                          cli['arguments'])