## Installation

1. Clone the repo.
2. Install PyYaml and Tabulate

~~~
pip install pyyaml
pip install tabulate
~~~

Note: If you don't have `pip`, then you can install `pip` via `easy_install` and then proceed to install PyYaml via `pip`.

~~~
easy_install pip
~~~

## Configuration

Make some [aliases](http://www.computerhope.com/unix/ualias.htm) for a dose of additional realism.

~~~
alias puppet-tool_name="python ~/path/to/repo/tool_name.py"
~~~

## Usage

The prototype supports as many of the commands as possible. See the design doc or the prototype help pages for more information.

#### Print 'Help' Pages

~~~
# via python
python tool_name.py -h

# via aliases
puppet-tool_name -h
~~~

## Resources

### Internal

### External
* [Docopt](http://docopt.org/)
