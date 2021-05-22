package main

import "testing"

func TestParseEchoRequest(t *testing.T) {
	req, err := parseEchoRequest([]byte("{\"kind\":\"create\",\"message\":\"hello\"}"))
	if err != nil {
		t.Fatal(err)
	}
	if req.Kind != "create" {
		t.Fatalf("unexpected Kind %q", req.Kind)
	}
	if req.Message != "hello" {
		t.Fatalf("unexpected Message %q", req.Message)
	}
}
