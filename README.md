# Commenter 🗨️

**Commenter** is a web server for everyone who wants to have commenting functionalities on their applications.

### What's the challenge? 💪

- Implementing a commenting system with nested threads and multiple levels of engagement, such as replies and reactions,
  presents challenges primarily around data structure, performance, and scalability;
- The data model must efficiently handle the complexity of relationships and hierarchies within comments. Performance
  optimization is key to ensure quick loading and posting of comments, especially as the volume of data grows;
- Ensuring the commenting system can scale to accommodate an increasing number of users and comments while maintaining
  fast response times is also a significant technical hurdle.

### Features ✨

- **Cloud-Native Architecture**: designed and dockerizable as a microservice for seamless integration into cloud
  ecosystems like Kubernetes (k8s) deployments, stateless making it inherently scalable and resilient.

- **Comprehensive API Suite**: a flexible and rich set of RESTful APIs designed with ergonomics in mind, facilitating a
  wide array of commenting scenarios for various digital content forms like blogs, news articles, and forums. It also
  supports social features, enabling users to express themselves through comments, threaded replies, and reactions with
  emojis.

- **Rusty**: Built in Rust, with performance and reliability. It remains fast and stable under demanding
  workloads.

- **MongoDB Powered**: Leveraging the flexibility and speed of MongoDB for data storage. We're also committed to
  broadening our database support, with plans to integrate with other databases, offering you the freedom to choose the
  best storage solution for your needs.

### Ideas 💡

Comment sections are structured as a tree, with:

![Comments Diagram](./comments.png)

- **Resource**: The digital content (posts, blogs, articles, forums) that comments are associated with.
- **Root Comment**: The initial comment level where users can begin threaded discussions.
- **Branch Comments**: Subsequent levels of threaded comments that stem from root comments or other branches.

### Requirements

All necessary configurations for the server and MongoDB connection parameters should be provided via environment
variables. Here's an example of the required environment variables for your development setup:

- `SERVER_HOST` (string): The IP address where the Commenter will run. Defaults to `127.0.0.1` if not set.
- `SERVER_PORT` (number): The port number on which the Commenter server will listen. Defaults to `7000` if not set.
- `MONGODB_CONNECTION_STRING` (string): The MongoDB connection string.
- `MONGODB_MAX_POOL_SIZE` (number, optional): The maximum number of connections in the MongoDB connection pool.

### Development MongoDB Setup 🛠️

```
cd dev 

docker build -t commenter-mongo:latest -f mongo.Dockerfile .

docker run -v commenter-mongo-data:/data/db -p 27017:27017 -d commenter-mongo:latest
```

To connect to this local MongoDB container, ensure your environment variables are set accordingly:

```
MONGODB_CONNECTION_STRING=mongodb://commenter:commenter@localhost:27017/commenter?authSource=admin

MONGODB_MAX_POOL_SIZE=100
```

### Spin up the commenter! 🚀

```
cargo run --package commenter --bin commenter
```

### If you doubt it, dockerise it! 🐳

```
docker build -t commenter:latest .

docker run -e SERVER_HOST=127.0.0.1 -e SERVER_PORT=7000 -e MONGODB_CONNECTION_STRING=mongodb://commenter:commenter@localhost:27017/commenter?authSource=admin -e MONGODB_MAX_POOL_SIZE=100 -p 7000:7000 commenter:latest
```

^^ this assumes you want to run the commenter container at local port `7000`

### API User Manuals 📘

| Action                              | HTTP Method | Endpoint                | Description                                                                                                           | Payload                                                                                                                                 |
|-------------------------------------|-------------|-------------------------|-----------------------------------------------------------------------------------------------------------------------|-----------------------------------------------------------------------------------------------------------------------------------------|
| Create a Root Comment               | `POST`      | `/root-comment/new`     | Creates a root-level comment associated with a specific resource ID.                                                  | `{ "resource_id": "Uuid string", "commenter_account_id": "Uuid string", "commenter_username": "string", "comment_text": "string" }`     |
| Retrieve All Comments               | `GET`       | `/comments/all`         | Retrieves all comments linked to a particular resource ID.                                                            | `{ "resource_id": "Uuid string" }`                                                                                                      |
| Retrieve Root Comments              | `GET`       | `/root-comments`        | Fetches all root-level comments for a given resource ID, with an optional limit on the results.                       | `{ "resource_id": "Uuid string", "limit": "optional u32" }`                                                                             |
| Create a Branch Comment             | `POST`      | `/branch-comment/new`   | Adds a branch comment stemming from a root or another branch comment.                                                 | `{ "branched_from": "Uuid string", "commenter_account_id": "Uuid string", "commenter_username": "string", "comment_text": "string" }`   |
| Retrieve Next-Level Branch Comments | `GET`       | `/branch-comments/next` | Obtains branch comments that are directly branching from the given comment ID, with an optional limit on the results. | `{ "branched_from": "Uuid string", "limit": "optional u32" }`                                                                           |
| Retrieve Subsequent Branch Comments | `GET`       | `/branch-comments/rest` | Retrieves all branch comments that follow after the comment specified by the given comment ID.                        | `{ "branched_from": "Uuid string" }`                                                                                                    |
| Update Comment Text                 | `POST`      | `/comment/update`       | Enables a user to edit the text of their previously posted comment.                                                   | `{ "comment_id": "Uuid string", "new_comment_text": "string" }`                                                                         |
| Delete a Comment                    | `POST`      | `/comment/delete`       | Removes a comment and all related branch comments.                                                                    | `{ "comment_id": "Uuid string" }`                                                                                                       |
| React to a Comment                  | `POST`      | `/reaction/new`         | Permits a user to react to a comment with an emoji.                                                                   | `{ "reactor_account_id": "Uuid string", "reactor_username": "string", "emoji_unicode": "string", "reacted_comment_id": "Uuid string" }` |
| Undo a Reaction                     | `POST`      | `/reaction/undo`        | Allows a user to remove their reaction from a comment.                                                                | `{ "reactor_account_id": "Uuid string", "reactor_username": "string", "emoji_unicode": "string", "reacted_comment_id": "Uuid string" }` |

### Contributors 👥

Java M (https://github.com/wjjmjh)

### Our Values 🌟

We have fundamental philosophy: Giving back more to the world than we take from it. We're deeply committed to being
active contributors, supporters, and sponsors within the open-source community.

If you have some ideas on Commenter, hey, just simply fork it, make a PR, our team will review your contribution and get
back to you as soon as possible!

🥇 Every contributor will be honorably mentioned, and their contributions will be immortalized in the git commit history
of this repository.
