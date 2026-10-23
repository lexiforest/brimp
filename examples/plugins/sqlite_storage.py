"""Explicit durable hostStorage backend. This is not a browser IndexedDB backend.

Usage: plugins.host_storage(backend_factory=lambda: SQLiteStorage(path))
Only JSON values are supported by this example; the default memory backend also
supports copied binary values. SQLite coordinates transactions across connections.
"""
import json
import sqlite3
import threading


class SQLiteStorage:
    def __init__(self, path):
        self.lock = threading.RLock()
        self.connection = sqlite3.connect(path, check_same_thread=False, isolation_level=None)
        self.connection.execute("CREATE TABLE IF NOT EXISTS kv (origin TEXT, key TEXT, value TEXT NOT NULL, PRIMARY KEY(origin,key))")

    def transaction(self, origin, operations):
        with self.lock:
            self.connection.execute("BEGIN IMMEDIATE")
            try:
                results = []
                for op in operations:
                    kind, key = op["type"], op.get("key")
                    if kind == "get":
                        row = self.connection.execute("SELECT value FROM kv WHERE origin=? AND key=?", (origin, key)).fetchone()
                        results.append(json.loads(row[0]) if row else None)
                    elif kind == "set":
                        value = json.dumps(op["value"], allow_nan=False)
                        self.connection.execute("INSERT INTO kv VALUES (?,?,?) ON CONFLICT(origin,key) DO UPDATE SET value=excluded.value", (origin, key, value))
                        results.append(None)
                    elif kind == "delete":
                        changed = self.connection.execute("DELETE FROM kv WHERE origin=? AND key=?", (origin, key)).rowcount
                        results.append(changed > 0)
                    elif kind == "clear":
                        self.connection.execute("DELETE FROM kv WHERE origin=?", (origin,))
                        results.append(None)
                    else:
                        raise ValueError("Invalid storage operation")
                self.connection.execute("COMMIT")
                return results
            except BaseException:
                self.connection.execute("ROLLBACK")
                raise

    def close(self):
        with self.lock:
            self.connection.close()
