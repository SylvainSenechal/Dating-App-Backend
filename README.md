# Dating-App
Run in prod : 
1. cargo build --release
2. export AWS_ACCESS_KEY_ID=...
3. export AWS_SECRET_ACCESS_KEY=...
4. Add jwt keys in src/configs/prod.toml
5. Install sqlite with math functions enabled :
- download sqlite autoconf
- tar -xvf sqlite-autoconf-*.tar.gz
- cd sqlite-autoconf-*
- ./configure --enable-math
- make
- sudo make install
6. Create db: cat databaseCreation.sql | sqlite3 love.db
7. Run: nohup sudo -E ./target/release/backend
- nohup : keep running after ssh closed
- sudo : using restricted port 80
- E : use env variables even in sudo mode 
