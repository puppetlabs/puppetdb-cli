#!/bin/sh

set -ue

if ! test "$#" -eq 1; then
    >&2 echo "usage: pod2man.sh <outpath>"
    exit 1
fi

which pod2man > /dev/null
test -f "man/puppet-db.pod"

pod2man_helper() {
    local manfile="$1"
    local outpath="$2"
    local center="$3"
    test -f "man/${manfile}.pod"
    pod2man --section 8 --release \
            --center "$center" \
            --name "$manfile" \
            "man/${manfile}.pod" "${outpath}/share/man/man8/${manfile}.8"
}

outpath="$1"
mkdir -p "${outpath}/share/man/man8"
pod2man_helper "puppet-db" "$outpath" "manages PuppetDB administrative tasks"
pod2man_helper "puppet-query" "$outpath" "queries PuppetDB data"
pod2man_helper "puppetdb_conf" "$outpath" "PuppetDB CLI configuration"
