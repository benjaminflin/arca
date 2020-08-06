CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
SELECT uuid_generate_v4();
CREATE TABLE users (
	id uuid DEFAULT uuid_generate_v4(),
  email VARCHAR NOT NULL,
  pass_hash VARCHAR NOT NULL,
  os_user VARCHAR,
  PRIMARY KEY (id)
);