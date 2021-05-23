
# Task scheduler implementation in Go and Rust

Feature requirements are:

1. A client sends a task request via HTTP endpoint, and such requests can be multiple and sent with concurrency.
2. The server generates a task ID for each request.
3. The server serialize requests into an apply wait queue with the task ID.
4. The applier receives the requested task from the queue.
5. The applier applies the task, and trigger complete event to notify the server.
6. The server responds to the client request via HTTP.

This is basically how [etcd](https://github.com/etcd-io/etcd) server works.

See https://github.com/rust-lang/wg-async-foundations/pull/209 for more discussions.

