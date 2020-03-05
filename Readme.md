# Hook-Recorder
Simple webhook recorder written in rust which persists records to a postgres database and has a display endpoint which shows the headers and body for the most recently recorded hook.

Warp is the web framework and Diesel is the ORM in use, metrics is used for a metrics facade with metrics_runtime providing the controller and exporter.

The webapp container is very lean, using a statically compiled rust binary and a scratch container to minimize footprint as much as possible.


# Deploying
Clone repository
```
git clone https://github.com/nharring/hook-recorder.git
```
Create external volume for postgres
```
docker volume create db-data
```
Launch stack
```
docker-compose -f "hook-recorder/docker-compose.yml" up -d
```

If you need to debug from inside the container change the Dockerfile to uncomment FROM ALPINE and comment FROM SCRATCH and rebuild.

An existing postgres installation can be used by modifying the environment vars in docker-compose.yml, if you use your own DB also be sure to remove the db service and the webapps dependency on it. You can also remove the backend bridge network in this case.
