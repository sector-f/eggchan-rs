INSERT INTO boards (name, description) VALUES
	('out', NULL),
	('diy', 'Do It Yourself')
;

INSERT INTO posts (board_id, reply_to, comment) VALUES
	(1, NULL, 'First post on out'),
	(2, NULL, 'First post on diy'),
	(1, 1, 'Reply to first post on out')
;
