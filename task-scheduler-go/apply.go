package main

import (
	"errors"
	"fmt"
	"math/rand"
	"time"
)

type Request struct {
	echoRequest *EchoRequest
}

type Applier interface {
	start()
	stop() error
	apply(req Request) (string, error)
}

func newApplier(requestTimeout time.Duration) Applier {
	memberID := rand.Uint64()
	return &applier{
		requestTimeout: requestTimeout,

		requestIDGenerator: newGenerator(memberID, time.Now().UnixNano()),
		notifier:           newNotifier(),

		requestCh: make(chan requestTuple, 1000),
		stopCh:    make(chan struct{}),
		doneCh:    make(chan struct{}),

		echoManager: newEchoManager(),
	}
}

// ref. https://github.com/etcd-io/etcd/blob/release-3.5/pkg/schedule
type applier struct {
	requestTimeout time.Duration

	requestIDGenerator Generator
	notifier           Notifier

	// for sending requests in the queue
	requestCh chan requestTuple

	stopCh chan struct{}
	doneCh chan struct{}

	echoManager EchoManager
}

type requestTuple struct {
	requestID uint64
	request   Request
}

func (ap *applier) start() {
	go func() {
		fmt.Println("starting applier async")
		for {
			select {
			case tup := <-ap.requestCh:
				reqID := tup.requestID
				req := tup.request
				switch {
				case req.echoRequest != nil:
					rs, err := ap.echoManager.apply(req.echoRequest)
					if err != nil {
						rs = fmt.Sprintf("failed to apply %v", err)
					}
					if err = ap.notifier.trigger(reqID, rs); err != nil {
						fmt.Printf("failed to trigger %v", err)
					}
				default:
				}
			case <-ap.stopCh:
				fmt.Println("received stop signal")
				ap.doneCh <- struct{}{}
				fmt.Println("signaled done")
				return
			}
		}
	}()
}

func (ap *applier) stop() error {
	fmt.Println("stopping applier")
	select {
	case ap.stopCh <- struct{}{}:
	case <-time.After(5 * time.Second):
		return errors.New("took too long to signal stop")
	}
	select {
	case <-ap.doneCh:
	case <-time.After(5 * time.Second):
		return errors.New("took too long to receive done")
	}
	fmt.Println("stopped applier")
	return nil
}

func (ap *applier) apply(req Request) (string, error) {
	reqID := ap.requestIDGenerator.next()
	respRx, err := ap.notifier.register(reqID)
	if err != nil {
		return "", err
	}

	select {
	case ap.requestCh <- requestTuple{requestID: reqID, request: req}:
	case <-time.After(ap.requestTimeout):
		if err = ap.notifier.trigger(reqID, fmt.Sprintf("failed to schedule %d in time", reqID)); err != nil {
			return "", err
		}
	}

	msg := ""
	select {
	case msg = <-respRx:
	case <-time.After(ap.requestTimeout):
		return "", errors.New("apply timeout")
	}

	return msg, nil
}
