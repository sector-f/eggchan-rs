# eggchan

`eggchan` is (the codename of) an imageboard backend.
It returns information about boards and threads in a JSON format, allowing different frontends to be written easily.

## To-Do

- [ ] Implement board categories
- [ ] Implement image uploads
- [ ] Add optional Name field to posts
- [ ] Make sure an error is returned if you try to reply to a post that isn't an OP (i.e. if posts.reply_to IS NOT NULL then error)
- [ ] Maybe write a SQL script to fill up an empty DB with some fixed test data
- [ ] Change all returned error messages to JSON
- [ ] Implement API users
- [ ] Create CLI program to query the board's database
  - [x] List boards
  - [ ] List threads
  - [ ] List API users

### HTTP Endpoints

- [x] Get list of all boards
- [x] Get list of threads on a given board
- [x] Get a given thread on a given board
- [x] Create a new thread on a given board
- [x] Reply to a given thread
- [ ] Add API user
