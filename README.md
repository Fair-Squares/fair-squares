<div align="center">
  <a href="https://discord.gg/5u3dxE49V5">
    <img alt="Discord" src="https://img.shields.io/discord/899662897003778139?label=Fair%20squares%20&logo=Discord&logoColor=red" />
  </a>
  <a href="https://twitter.com/FairSquares">
    <img alt="Twitter Follow Fair Squares" src="https://twitter.com/FairSquares"/>
  </a>
</div>


# Fair Squares (FS)
## Description
FS connects supply and demand of house-owners & renters and houses & investors. Our motive is that we want to create an affordable housing market. Investors of a house get a social return while renters can have cheaper housing. 

It removes the financial barrier of investing in real estate for investors that don't have the means to fully invest in a house themselves. In between the end-users, there is coordination taking place between different stakeholders to achieve the desired outcome. This is where the runtime and the logic of all pallets come together, orchestrating while adhering to strict rules set for a equitable system.  

FS can be positions as a programmed DAO that has the financial tools housing-fund that is programmed to bid on houses as they have been succesfully onboarded. The onboarding is achieved by verfied actors such as a fullfilling tasks to get the truth on-chain. The real world actors the following real-world actors to fullfill tasks and get paid out by the digital society. 

We are zooming much more on the problem definition, stakeholders and the solution in our paper on our [website](https://fair-squares.nl/). To learn more and get in touch with us, please join our [discord channel FS](https://discord.gg/5u3dxE49V5)

</br>

## Run & build
### Running locally
1. complete the [basic Rust setup instructions](./docs/rust-setup.md).
1. `cargo run  --release -- --dev --tmp` in the root of the fs-node repo.
### Build locally

The `cargo build` command will perform an initial build. 

```sh
cargo build --release
```
The binary will be present in create the binary in `./target/release/fs-node` if not other argument is passed. 

### Docker build & run
We added a [Dockerfile](https://github.com/Fair-Squares/fair-squares/blob/main/Dockerfile) in the repo, you can build an image yourself with the following commmand `docker build .`

### Docker images
//


### Run in Docker in linux

First, install [Docker](https://docs.docker.com/get-docker/) and
[Docker Compose](https://docs.docker.com/compose/install/).

Then run the following command to start a single node development chain.
```bash
./scripts/docker_run.sh
```
The script above will need a folder in the root of this project called `.local` , you will have to create this folder yourself.


This command will firstly compile your code, and then start a local development network. You can also replace the default command
(`cargo build --release && ./target/release/fs-node --dev --ws-external`)
by appending your own. A few useful ones are as follow.

### Connect with Polkadot-Js apps front-end

Once the node template is running locally, you can connect it with **Polkadot-JS Apps** front-end to interact with your chain. [Polkadot.js](https://polkadot.js.org/apps/#/explorer?rpc=ws://localhost:9944) connects a front-end is the app that can interact with the node by means of extensics calls and can read the chain state of the blockchain. Click [here](https://polkadot.js.org/apps/#/explorer?rpc=ws://localhost:9944) to connect to the local blockchain

## Run all tests

```
cargo test
```