CREATE TABLE users (
	id uuid DEFAULT uuid_generate_v4(),
  email VARCHAR NOT NULL,
  pass_hash VARCHAR NOT NULL,
  PRIMARY KEY (id)
);