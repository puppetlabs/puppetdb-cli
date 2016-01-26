#!/usr/bin/env python

import os
import re
import sys
import time
import yaml
from random import random, randint
from pprint import pprint
from docopt import docopt, DocoptExit

CLI_YAML_DIR = 'yaml'
CLI_YAML_EXT = '.yaml'

def add_color(msg, color):
    """Return the supplied msg in the given ansii color."""

    color_codes = {
        'red': '\033[0;31m',
        'green': '\033[0;32m',
        'yellow': '\033[0;33m',
        'blue': '\033[0;34m',
        'purple': '\033[0;35m',
        'cyan': '\033[0;36m',
        'gray': '\033[1;30m',
        'white': '\033[1;37m',
        'none': '\033[0;37m'
    }

    try:
        color_codes[color]
    except KeyError:
        color = 'none'

    return "{color}{msg}\033[0m".format(msg=msg, color=color_codes[color])


def print_color(msg, color):
    """Prints the supplied msg in the given ansii color."""

    # print() tends to print empty spaces for ansii codes
    sys.stdout.write(add_color(msg, color))


def print_log_message(msg,level=False):
    if level == 'error':
        print_color('ERROR    ','red'),
    if level == 'warning':
        print_color('WARNING  ','yellow'),
    if level == 'notice':
        print_color('NOTICE   ','white'),
    if level == 'info':
        print_color('INFO     ','green'),
    if level == 'debug':
        print_color('DEBUG    ','blue'),
    else:
        print msg

def show_progress(duration=0,msg='running',exit_code=0):
    print msg,
    for i in range(duration):
        print '\b.',
        time.sleep(0.2)
        sys.stdout.flush()
    if exit_code == 0:
        print_color(' Done!\n','green')
    elif exit_code == 1:
        print_color(' Failed!\n','yellow')
    elif exit_code == 2:
        print_color(' Failed!\n','red')
    elif exit_code == 3:
        print_color(' \n','blue')


def add_tab(s, spaces=4):
    """Return supplied string with a tab of spaces size."""

    return ' ' * spaces + s


def pause_time(min_sec=1, max_sec=5):
    """Pause the CLI for a random period of time."""

    sleep(random() * float(randint(min_sec, max_sec)))


def get_cli_from_command(command):
    """Returns a docopt CLI command and arguments."""

    try:
        cli = load_yaml(command)
    except IOError as e:
        sys.exit("'{0}' file not found. Exiting.".format(e))

    try:
        arguments = docopt(cli['docopt'])
    except KeyError as e:
        sys.exit("'{0}' usage statement not found. Exiting.".format(e))

    # DocoptExit is raised when user arguments do not match what docopt expects
    except DocoptExit as e:
        sys.exit(e)    # prints docopt usage statement

    return {'cli': cli, 'arguments': arguments}


def load_config(config_file='config'):
    """Returns a CLI config object for accessing CLI stage."""

    try:
        config = load_yaml(config_file)
    except IOError:
        config = {}     # no config file found, create config object

    return config


def save_config(config, config_file='config'):
    """Saves the supplied config object to the config yaml file."""

    try:
        with open(get_yaml_filename(config_file), 'w') as yaml_file:
            yaml.dump(config, yaml_file, default_flow_style=False)
    except IOError as e:
        sys.exit("Unable to save CLI config file with error: {0}".format(e))


def get_yaml_filename(filename):
    """Returns a fully qualified yaml filename for the supplied filename."""

    return os.path.join(os.path.dirname(os.path.realpath(__file__)),
                        CLI_YAML_DIR,
                        '{0}{1}'.format(filename, CLI_YAML_EXT))


def load_yaml(yaml_name):
    """Returns a python yaml object from a supplied yaml file name."""

    return yaml.safe_load(open(get_yaml_filename(yaml_name), 'r'))


def print_cli_arguments(arguments):
    """Print the arguments received by the docopt CLI."""

    print("Docopt CLI Arguments:\n")
    pprint(arguments)
    print("\nDocopt CLI Feedback:\n")


def exit_cli(name=''):
    """Exit the CLI."""

    sys.exit(("\nQuitting {name} and disconnecting from the service ...\n").format(name=name))
