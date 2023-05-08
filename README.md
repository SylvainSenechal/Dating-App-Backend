# Dating-App
prod : 
- cargo build --release
- export AWS_ACCESS_KEY_ID=...
- export AWS_SECRET_ACCESS_KEY=...
- cat databaseCreation.sql | sqlite3 love.db
- nohup sudo -E ./target/release/backend

nohup : keep running after ssh closed
sudo : using restricted port 80
E : use env variables even in sudo mode 
