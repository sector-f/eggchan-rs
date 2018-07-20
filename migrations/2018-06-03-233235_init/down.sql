-- This file should undo anything in `up.sql`
DROP TRIGGER update_postnum_table ON boards;
DROP FUNCTION make_board_postnum;
DROP TRIGGER update_postnum ON posts;
DROP FUNCTION post_num_trigger;
DROP TABLE posts;
DROP TABLE images;
DROP TABLE board_postnum;
DROP TABLE boards;
DROP TABLE categories;
