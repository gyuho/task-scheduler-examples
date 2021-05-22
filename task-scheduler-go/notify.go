package main

import (
	"fmt"
	"sync"
)

// ref. https://github.com/etcd-io/etcd/tree/release-3.5/pkg/wait
type Notifier interface {
	register(id uint64) (<-chan string, error)
	trigger(id uint64, x string) error
}

func newNotifier() Notifier {
	return &notifier{
		requests: make(map[uint64]chan string),
	}
}

type notifier struct {
	mu       sync.RWMutex
	requests map[uint64]chan string
}

func (ntf *notifier) register(id uint64) (<-chan string, error) {
	fmt.Println("registering", id)
	ntf.mu.Lock()
	defer ntf.mu.Unlock()
	ch := ntf.requests[id]
	if ch != nil {
		return nil, fmt.Errorf("dup id %x", id)
	}

	ch = make(chan string, 1)
	ntf.requests[id] = ch
	fmt.Println("registered", id)
	return ch, nil
}

func (ntf *notifier) trigger(id uint64, x string) error {
	fmt.Println("triggering", id)
	ntf.mu.Lock()
	ch, ok := ntf.requests[id]
	if ch == nil || !ok {
		ntf.mu.Unlock()
		return fmt.Errorf("request ID %d not found", id)
	}
	delete(ntf.requests, id)
	ntf.mu.Unlock()

	ch <- x
	close(ch)
	fmt.Println("triggered", id)
	return nil
}
