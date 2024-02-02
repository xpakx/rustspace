create table "posts" (
	id serial primary key,
	title text,
	content text,
	created_at timestamptz not null default now(),
	updated_at timestamptz not null default now(),
	user_id int not null,
	constraint fk_user_id
		foreign key(user_id)
		references users(id)
);

create trigger update_timestamp
before update
on "posts"
for each row
	execute procedure update_modified_column();

