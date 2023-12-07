create table "users" (
	id serial primary key,
	screen_name text unique not null,
	email text unique not null,
	password text not null,
	created_at timestamptz not null default now(),
	updated_at timestamptz not null default now()
	);

create or replace function update_modified_column()
returns trigger as $$
begin
	new.updated_at = now();
	return new;
end;
$$ language 'plpgsql';

create trigger update_timestamp
	before update
	on "users"
	for each row
	execute procedure update_modified_column();

