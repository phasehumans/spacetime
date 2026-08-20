#!/usr/bin/env bash
set -e

apt-get update && DEBIAN_FRONTEND=noninteractive apt-get install -y sqlite3

mkdir -p /root/ecommerce
sqlite3 /root/ecommerce/store.db <<'EOF'
CREATE TABLE users (
    id INTEGER PRIMARY KEY,
    email TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE orders (
    id INTEGER PRIMARY KEY,
    user_id INTEGER,
    status TEXT NOT NULL,
    total REAL,
    FOREIGN KEY(user_id) REFERENCES users(id)
);

INSERT INTO users (id, email) VALUES (1, 'alice@example.com'), (2, 'bob@example.com');
INSERT INTO orders (id, user_id, status, total) VALUES (1, 1, 'pending', 99.50), (2, 2, 'completed', 150.00);
EOF
