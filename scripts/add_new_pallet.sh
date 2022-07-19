#!/usr/bin/env bash
# This script is meant to be run on Unix/Linux based systems
set -e
set -u
set -o pipefail

space=""
tab="tolp"
file_lib="runtime/src/lib.rs"

usage ()
{
    # Display usage of the script
    echo " Usage: $0 name Name"
    echo " eg: $0 housing_fund HousingFund"
}

# $1 old text
# $2 new text
# $3 file name
replace ()
{
    sed -i "s/$1/$2/g" $3
}

## $1 old_text
insert_new_entry()
{
    local_old_text="### add new $1"
    local_replace_tag="### add a new $1"
    local_config_pallet_line="$tab\t\"pallet-$pallet_name/$1\","
    local_new_text="$local_config_pallet_line\n\t$local_replace_tag"
    if ! grep -q "$local_config_pallet_line" runtime/Cargo.toml
    then
        sed -i "/$local_old_text/a $local_new_text" runtime/Cargo.toml
        sed -i "/$local_old_text/d" runtime/Cargo.toml
        sed -i "s/$local_replace_tag/$local_old_text/g" runtime/Cargo.toml
    fi
}

## $1 keyword replacement
## $2 tabulation
replace_librs() 
{
    local_old_text="\/\/ flag add pallet $1"
    local_replace_tag="\/\/ flag add a pallet $1"
    local_new_text="$config_pallet_line\n$2$local_replace_tag"
    if ! grep -q "$config_pallet_line" $file_lib
    then
        sed -i "/$local_old_text/a $local_new_text" $file_lib
        sed -i "/$local_old_text/d" $file_lib
        sed -i "s/$local_replace_tag/$local_old_text/g" $file_lib
    fi
}

if [ "$#" -ne 2 ]
then
    usage
    exit
fi

pallet_name=$1
pallet_module_name=$2

# echo " copy template folder in pallets folder"
if [ -d "pallets/pallet_template" ]
then
    rm -rf "pallets/pallet_template"
elif [ -d "pallets/$pallet_name" ]
then
    echo " pallet $pallet_name already exists"
    exit
fi

cp -r assets/pallet_template pallets/

# echo " rename template folder to $pallet_name "
mv pallets/pallet_template pallets/$pallet_name

# echo " update $pallet_name/Cargo.toml "
pallet_folder=pallets/$pallet_name
replace "pallet-template" "pallet-$pallet_name" $pallet_folder/Cargo.toml

# echo " update $pallet_name/src/mock.rs "
replace "pallet_template" "pallet_$pallet_name" $pallet_folder/src/mock.rs

old_text="TemplateModule"
new_text="${pallet_module_name}Module"
replace $old_text $new_text $pallet_folder/src/mock.rs

# echo " update $pallet_name/src/test.rs "
replace $old_text $new_text $pallet_folder/src/tests.rs

# echo " update $pallet_name/src/benchmarking.rs "
replace "Template" "${pallet_module_name}" $pallet_folder/src/benchmarking.rs



# echo " update runtime/Cargo.toml "
old_text="### add new pallet config"
replace_tag="### add a new pallet config"
config_pallet_line="pallet-$pallet_name = { version = \"4.0.0-dev\", default-features = false, path = \"../pallets/$pallet_name\" }"
new_text="$config_pallet_line\n$replace_tag"
if ! grep -q "$config_pallet_line" runtime/Cargo.toml
then
    sed -i "/$old_text/a $new_text" runtime/Cargo.toml
    sed -i "/$old_text/d" runtime/Cargo.toml
    sed -i "s/$replace_tag/$old_text/g" runtime/Cargo.toml
fi

declare -a StringArray=("std" "runtime-benchmarks" "try-runtime" )
for val in ${StringArray[@]}; do
   insert_new_entry $val
done

sed -i "s/$tab/$space/g" runtime/Cargo.toml

# echo " update runtime/src/lib.rs "

config_pallet_line="pub use pallet_$pallet_name;"
replace_librs "use" ""

weight_line="type WeightInfo = pallet_$pallet_name::weights::SubstrateWeight<Runtime>;"
config_pallet_line="impl pallet_$pallet_name::Config for Runtime {\n\ttype Event = Event;\n\t$weight_line\n}\n"
replace_librs "config" ""

config_pallet_line="$tab\t\t${pallet_module_name}Module: pallet_$pallet_name,"
replace_librs "runtime" "\t\t"

config_pallet_line="$tab\t\t[pallet_$pallet_name, ${pallet_module_name}Module]"
replace_librs "bench_macro" "\t\t"

config_pallet_line="$tab\t\t\tadd_benchmark!(params, batches, pallet_$pallet_name, ${pallet_module_name}Module);"
replace_librs "benchmark" "\t\t\t"

sed -i "s/$tab/$space/g" $file_lib
