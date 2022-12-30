CREATE EXTENSION "uuid-ossp";
-- Add migration script here
CREATE TABLE master_book(
  master_book_id uuid primary key default uuid_generate_v1mc(),
  author text null,
  title text null,
  lccn text null,
  isbn text null,
  publish_date text null,
  status text null,
  create_at timestamptz not null default now(),
  updated_at timestamptz
);

