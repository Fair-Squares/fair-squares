#!/usr/bin/env bash
# This script is meant to be run on Unix/Linux based systems
set -e
set -u
set -o pipefail

# Set variables
build="false"
bench="false"
pallet_name="";
steps=100
repeat=40
all_pallets="false"
correct_input="false"

usage ()
{
    # Display usage of the script
     echo " Usage: $0 -[b|t|h|i] <pallet_name> [<steps>] [<repeat>]"
     echo " Execute $0 -h for more information"
}

help()
{
    # Display Help
    echo "Generate weight.rs file from benchmarking.rs for a specified pallet."
    echo
    echo "Syntax: "
    echo "    script [-b|t|h|i] <pallet_name> [<steps>] [<repeat>]"
    echo "    script -a[|b|t] [<steps>] [<repeat>]"
    echo "Default:"
    echo "    steps=$steps, repeat=$repeat"
    echo "Options:"
    echo "b     Execute 'cargo build --release --features runtime-benchmarks'."
    echo "t     Execute 'cargo test -p 'pallet_name' --features runtime-benchmarks'."
    echo "a     Execute for all pallets."
    echo "h     Print this Help."
    echo "i     Print the list of pallets."
    echo
}

if [ "$#" -eq 1 ]
then
    if [[ "$1" =~ ^[A-Za-z_]+ ]]
    then
        pallet_name=$1
        correct_input="true"
    elif [[ "$1" == -*a* ]]
    then
        correct_input="true"
    fi
elif [ "$#" -eq 2 ]
then
    if [[ "$1" == -*a* ]]
    then
        if [[ "$2" =~ ^[0-9]+ ]]
        then
            steps=$2
            correct_input="true"
        fi
    elif [[ "$1" == -* ]]
    then
        if [[ "$2" =~ ^[A-Za-z_]+ ]]
        then
            pallet_name=$2
            correct_input="true"
        fi
    elif [[ "$1" =~ ^[A-Za-z_]+ ]]
    then
        if [[ "$2" =~ ^[0-9]+ ]]
        then
            pallet_name=$1
            steps=$2
            correct_input="true"
        fi
    fi
elif [ "$#" -eq 3 ]
then
    if [[ "$1" == -*a* ]]
    then
        if [[ "$2" =~ ^[0-9]+ ]]
        then
            if [[ "$3" =~ ^[0-9]+ ]]
            then
                steps=$2
                repeat=$3
                correct_input="true"
            fi
        fi
    elif [[ "$1" == -* ]]
    then
        if [[ "$2" =~ ^[A-Za-z_]+ ]]
        then
            if [[ "$3" =~ ^[0-9]+ ]]
            then
                pallet_name=$2
                steps=$3
                correct_input="true"
            fi
        fi
    elif [[ "$1" =~ ^[A-Za-z_]+ ]]
    then
        if [[ "$2" =~ ^[0-9]+ ]]
        then
            if [[ "$3" =~ ^[0-9]+ ]]
            then
                pallet_name=$1
                steps=$2
                repeat=$3
                correct_input="true"
            fi
        fi
    fi
elif [ "$#" -eq 4 ]
then
    if [[ "$1" == -*a* ]]
    then
        correct_input="false"
    elif [[ "$1" == -* ]]
    then
        if [[ "$2" =~ ^[A-Za-z_]+ ]]
        then
            if [[ "$3" =~ ^[0-9]+ ]]
            then
                if [[ "$4" =~ ^[0-9]+ ]]
                then
                    pallet_name=$2
                    steps=$3
                    repeat=$4
                    correct_input="true"
                fi
            fi
        fi
    fi
fi

# determine the options given
while getopts ":abthhelpi" option; do
   case $option in
      a)
         all_pallets="true";;
      i)
         array=(pallets/*/)
         shopt -s extglob
         for elt in "${array[@]}"; 
         do 
            dir=${elt%%+(/)}
            dir=${dir##*/}
            dir=${dir:-/}
            echo "${dir}"
         done
         exit;;
      b) 
         build="true";;
      t) 
         bench="true";;
      h|help)
         help
         exit;;
      \?) # Invalid option
         echo "Error: Invalid option"
         exit;;
   esac
done

if [ $correct_input = "false" ]
then
    usage
    exit
fi

if [ $build = "true" ]
then
   cargo build --release --features runtime-benchmarks
    # echo "cargo build --release --features runtime-benchmarks"
fi

if [ $bench = "true" ]
then
    if [ $all_pallets = "true" ]
    then
        cargo test --features runtime-benchmarks
        # echo "cargo test --features runtime-benchmarks"
    else
        cargo test -p pallet-$pallet_name --features runtime-benchmarks
        # echo "cargo test -p pallet-$pallet_name --features runtime-benchmarks"
    fi
fi

if [ $all_pallets = "true" ]
then
    # echo "Generating weights.rs files... with steps=$steps and repeat=$repeat"
    array=(pallets/*/)
    shopt -s extglob
    for elt in "${array[@]}"; 
    do 
        dir=${elt%%+(/)}
        dir=${dir##*/}
        dir=${dir:-/}
        ./target/release/fs-node benchmark pallet \
            --chain dev \
            --execution=wasm \
            --wasm-execution=compiled \
            --pallet pallet_$dir \
            --extrinsic '*' \
            --steps $steps \
            --repeat $repeat \
            --output pallets/$dir/src/weights.rs \
            --template assets/frame-weight-template.hbs
    done
else
# Execute command to generate the weight.rs file for the pallet
    ./target/release/fs-node benchmark pallet \
        --chain dev \
        --execution=wasm \
        --wasm-execution=compiled \
        --pallet pallet_$pallet_name \
        --extrinsic '*' \
        --steps $steps \
        --repeat $repeat \
        --output pallets/$pallet_name/src/weights.rs \
        --template assets/frame-weight-template.hbs

    # echo "Generating weights.rs file... with steps=$steps and repeat=$repeat"
fi