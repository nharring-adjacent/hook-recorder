# Hook-Recorder

[![Build Status](https://travis-ci.com/nharring/hook-recorder.svg?branch=master)](https://travis-ci.com/nharring/hook-recorder) [![codecov](https://codecov.io/gh/nharring/hook-recorder/branch/master/graph/badge.svg)](https://codecov.io/gh/nharring/hook-recorder)

Simple webhook recorder written in rust which persists records to a postgres database and has a display endpoint which shows the headers and body for the most recently recorded hook.

Warp is the web framework and Diesel is the ORM in use, metrics is used for a metrics facade with metrics_runtime providing the controller and exporter.

The webapp container is very lean, using a statically compiled rust binary and a scratch container to minimize footprint as much as possible.

## Deploying

Clone repository

```bash
git clone https://github.com/nharring/hook-recorder.git
```

Create external volume for postgres

```bash
docker volume create db-data
```

Launch containers

```bash
docker-compose -f "hook-recorder/docker-compose.yml" up -d
```

If you need to debug from inside the container change the Dockerfile to uncomment FROM ALPINE and comment FROM SCRATCH and rebuild.

An existing postgres installation can be used by modifying the environment vars in docker-compose.yml, if you use your own DB also be sure to remove the db service and the webapps dependency on it. You can also remove the backend bridge network in this case.

## Developing

Fork and/or clone the repository and then setup the diesel cli (see <http://diesel.rs/guides/getting-started/> for more info on getting started with Diesel.)

```bash
cargo install diesel_cli
```

Export DATABASE_URL (or use .env) with values appropriate to your dev environment

```bash
bash> export DATABASE_URL=postgres://user:pass@host/db
powershell> $env:DATABASE_URL='postgres://user:pass@host/db'
```

Run diesel database setup (we don't need initial setup since we have migrations from the git repo)

```bash
diesel database setup
```
