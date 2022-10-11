#!/usr/bin/env bash
# This script is meant to be run on Unix/Linux based systems
set -e
set -u
set -o pipefail

template="polkadot-v"
old_version=""
new_version=""
file_name=""

usage ()
{
    # Display usage of the script
     echo " Usage: $0 old_version new_version"
}

display()
{
    ls -l $1
}

replace()
{
    if [[ $OSTYPE == 'darwin'* ]]; then
        gsed -i "s/$1/$2/g" $3
    else
        sed -i "s/$1/$2/g" $3
    fi
}

if [ "$#" -ne 2 ]
then
    usage
    exit
fi

old_version=$1
new_version=$2

old_text="${template}${old_version}"
new_text="${template}${new_version}"
# echo "old_text: ${old_text}"
# echo "new_text: ${new_text}"

replace $old_text $new_text assets/pallet_template/Cargo.toml
echo " updated assets/pallet_template/Cargo.toml"
replace $old_text $new_text node/Cargo.toml
echo " updated node/Cargo.toml"
replace $old_text $new_text runtime/Cargo.toml
echo " updated runtime/Cargo.toml"

array=(pallets/*/)
shopt -s extglob
for elt in "${array[@]}";
do
    dir=${elt%%+(/)}
    dir=${dir##*/}
    dir=${dir:-/}
    file_name=$(printf %q "pallets/${dir}/Cargo.toml")
    replace $old_text $new_text $file_name
    echo " updated $file_name"
done
