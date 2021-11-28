# werewolf-web-services
This is api services for repo [werewolf-web-frontend](https://github.com/Sherly1001/werewolf-web-frontend)

## database
### docker
Config database in [docker-compose.yml](https://github.com/Sherly1001/werewolf-web-services/blob/main/docker-compose.yml)  
Add `DATABASE_URL=postgres://actix:actix@localhost/actix` environment variable to `.env`  
Run following commands:
```sh
sudo docker-compose up -d
diesel migration run
```

### orther postgres database
Add your database url with environment variable `DATABASE_URL` and push it into file `.env` and run:
```sh
diesel migration run
```

## run
### local machine
To run on localhost just run:
```sh
cargo run
```

### deployment
Build project:
```sh
cargo build --release
```

Then start services with `./target/release/werewolf_services` binary file.
