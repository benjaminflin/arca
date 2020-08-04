CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
SELECT uuid_generate_v4();
CREATE TABLE users (
	id uuid DEFAULT uuid_generate_v4(),
  email VARCHAR NOT NULL,
  pass_hash VARCHAR NOT NULL,
  os_user VARCHAR NOT NULL,
  PRIMARY KEY (id)
);

CREATE OR REPLACE FUNCTION os_user_default()
  RETURNS trigger
  LANGUAGE plpgsql AS
$func$
BEGIN
  IF NEW.os_user IS NULL THEN 
    NEW.os_user := NEW.id;
  END IF;
  RETURN NEW;
END
$func$;

CREATE TRIGGER user_insert 
BEFORE INSERT OR UPDATE ON users 
FOR EACH ROW EXECUTE PROCEDURE os_user_default();