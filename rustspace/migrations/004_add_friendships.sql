create table "friendships" (
	id serial primary key,
	created_at timestamptz not null default now(),
	accepted boolean not null default false,
	rejected boolean not null default false,
	accepted_at timestamptz,
	user_id int not null,
	friend_id int not null,
	constraint fk_user_id
		foreign key(user_id)
		references users(id),
	constraint fk_friend_id
		foreign key(friend_id)
		references users(id)
);
