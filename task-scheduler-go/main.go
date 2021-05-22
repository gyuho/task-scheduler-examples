// Simple task scheduler and applier, inspired by etcd server.
package main

import (
	"flag"
	"time"
)

func main() {
	listenerPort := flag.Uint64("listener-port", 3000, "listener port")
	requestTimeoutSeconds := flag.Uint64("request-timeout-seconds", 5, "request timeout in seconds")

	srv := newHandler(*listenerPort, time.Duration(*requestTimeoutSeconds)*time.Second)
	srv.start()
}
