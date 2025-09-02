CREATE TABLE IF NOT EXISTS publications (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  url TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS articles (
  id TEXT PRIMARY KEY,
  publication_id TEXT NOT NULL,
  title TEXT NOT NULL,
  link TEXT NOT NULL,
  published_time TEXT NOT NULL,
  formatted_published_time TEXT NOT NULL,
  image TEXT,
  FOREIGN KEY (publication_id) REFERENCES publications(id)
);
