package main

import (
	"encoding/json"
	"fmt"
	"sync"
)

type EchoRequest struct {
	Kind    string `json:"kind"`
	Message string `json:"message,omitempty"`
}

func parseEchoRequest(d []byte) (req EchoRequest, err error) {
	err = json.Unmarshal(d, &req)
	return req, err
}

type EchoManager interface {
	apply(req *EchoRequest) (string, error)
}

func newEchoManager() EchoManager {
	return &echoManager{
		mu: sync.RWMutex{},
	}
}

type echoManager struct {
	mu sync.RWMutex
}

func (ea *echoManager) apply(req *EchoRequest) (string, error) {
	fmt.Println("applying echo request")
	ea.mu.Lock()
	defer ea.mu.Unlock()
	switch req.Kind {
	case "create":
		return fmt.Sprintf("SUCCESS create %q", req.Message), nil
	case "delete":
		return fmt.Sprintf("SUCCESS delete %q", req.Message), nil
	default:
		return "", fmt.Errorf("unknown request %q", req)
	}
}
