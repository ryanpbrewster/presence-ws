# Presence via websocket tracking

This is extension of the
[actix chat example](https://github.com/actix/actix/tree/master/examples/chat)


# Enable network access to port 8080
Any instance with the tag `http-server` will allow incoming traffic on port 8080.
```
gcloud compute firewall-rules create allow-http \
    --allow tcp:8080 --target-tags http-server
```

# Build and publish the server image
We'll be building this image:
```
export IMAGE_NAME="gcr.io/rpb-dev/presence-ws-server:$(git rev-parse HEAD)"
```

You can build it locally and push it, or build it via Cloud Build.

## Locally
```
docker build -t $IMAGE_NAME .
docker push $IMAGE_NAME
```

You may have to run
```
gcloud auth configure-docker
```
to get permissions to work (or give up on life and use `gcloud docker -- push $IMAGE_NAME`, which is deprecated).

## Cloud Build
```
gcloud builds submit --tag=$IMAGE_NAME --machine-type=n1-highcpu-8
```

# Spin up a server with the image

```
gcloud compute instances create-with-container presence-2 \
  --container-image $IMAGE_NAME \
  --machine-type=f1-micro \
  --zone=us-west1-b \
  --tags=http-server
```
