#!/bin/bash -e

<<COMMENT
curl -X POST -L http://localhost:3000/echo -d '{"kind": "create", "message": "hello"}'

# output
SUCCESS create hello
COMMENT

cargo run
