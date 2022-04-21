# P8e Contract + SDK Example

This project contains a complete example of how to create p8e contracts and execute them using Provenance
Blockchain's [p8e scope SDK](https://github.com/provenance-io/p8e-scope-sdk).

## Prerequisites

- Ability to download from GitHub container registry ([guide](https://docs.github.com/en/packages/working-with-a-github-packages-registry/working-with-the-container-registry))
- [Docker](https://docs.docker.com/get-started/)
- [Kotlin](https://kotlinlang.org/docs/home.html)

## Running the Example

If you have previously set up the local environment, you can skip step 2 if you haven't done one of the following:

- cleared the docker volumes
- changed the contract project

### 1. Spin Up the Local Environment

Our local environment relies on docker and the following images:

- [Provenance Blockchain](https://hub.docker.com/r/provenanceio/provenance)
- [Provenance Object Store](https://github.com/provenance-io/object-store/pkgs/container/object-store)
- [Postgres](https://hub.docker.com/_/postgres)

```shell
docker-compose -f ./docker/docker-compose.yaml up -d
```

__Note__: If this fails when pulling object store, check that you can pull images from GitHub container registry by following the guide linked under [Prerequisites](#prerequisites)

### 2. Publish the Contracts

Uses the [p8e-gradle-plugin](https://github.com/provenance-io/p8e-gradle-plugin) to digest the files included in the [contract](contract) project
and publish them as a contract to the locally running Provenance Blockchain instance.

```shell
source docker/env/bootstrap.env && ./gradlew p8eClean p8eBootstrap --info
```

### 3. Run the Example

This step is can be run any number of times while the local docker environment is up.  The example creates an example
[scope](https://docs.provenance.io/modules/metadata-module#scope-data-structures), which contains data about a fake 
loan that has been originated for a borrower.  After populating the scope with loan data, it showcases how to use the 
Contract Execution Environment to update the data by attaching a servicer to that loan.  This example effectively 
memorializes the new loan and servicer on the Provenance Blockchain, creating an immutable record of changes that may
occur during a loan's lifecycle.

```shell
./gradlew application:run
```

### 4. Check the Results on Provenance Blockchain

The example has put some data on our local chain, however, you might want to take a look at what is actually on chain.  Try out the following commands to see what is actually on chain.

```shell
# get a transaction
docker exec -it provenance provenanced query tx <tx hash>

# get a scope
docker exec -it provenance provenanced query metadata scope --include-sessions --include-records <scope id>
```

### 5. Spin Down the Local Environment

When you're finished

```shell
# allows you to spin up and rerun the example without step 2
docker-compose -f ./docker/docker-compose.yaml down

# clears the volumes which requires all steps to rerun the example
docker-compose -f ./docker/docker-compose.yaml down -v
```
