package main

import (
	"testing"
	"time"
)

func TestNotifier(t *testing.T) {
	ntf := newNotifier()
	ch, err := ntf.register(100)
	if err != nil {
		t.Fatal(err)
	}
	if err = ntf.trigger(100, "success"); err != nil {
		t.Fatal(err)
	}
	select {
	case msg := <-ch:
		if msg != "success" {
			t.Fatalf("unexpected message %q", msg)
		}
	case <-time.After(time.Second):
	}

	if err = ntf.trigger(100, "success"); err == nil {
		t.Fatal("expected error for trigger")
	}
}
