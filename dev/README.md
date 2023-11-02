### run local mongodb instance

```
docker build -t commenter-mongo:latest -f mongo.Dockerfile .

docker run -v commenter-mongo-data:/data/db -p 27017:27017 -d commenter-mongo:latest
```
