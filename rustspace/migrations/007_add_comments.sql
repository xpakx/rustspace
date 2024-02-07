create table "comments" (
	id serial primary key,
	content text,
	created_at timestamptz not null default now(),
	updated_at timestamptz not null default now(),
	user_id int not null,
	post_id int not null,
	constraint fk_user_id
		foreign key(user_id)
		references users(id),
	constraint fk_post_id
		foreign key(post_id)
		references posts(id)
);

create trigger update_timestamp
before update
on "comments"
for each row
	execute procedure update_modified_column();

