create table "profiles" (
	id serial primary key,
	gender text,
	city text,
	description text,
	real_name text,
	created_at timestamptz not null default now(),
	updated_at timestamptz not null default now(),
	user_id int not null,
	constraint fk_user_id
		foreign key(user_id)
		references users(id)
);

create trigger update_timestamp
before update
on "profiles"
for each row
	execute procedure update_modified_column();

