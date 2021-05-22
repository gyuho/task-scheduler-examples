package main

import (
	"fmt"
	"testing"
	"time"
)

func TestGenerator(t *testing.T) {
	gen := newGenerator(1, time.Now().UnixNano())
	fmt.Println(gen.next())
	fmt.Println(gen.next())
}

func TestGeneratorNext(t *testing.T) {
	dur := time.Duration(0x3456) * time.Millisecond
	gen := newGenerator(0x12, dur.Nanoseconds())
	id := gen.next()
	if id != 0x12000000345600 {
		t.Fatalf("unexpected id %x", id)
	}
	for i := 0; i < 1000; i++ {
		id2 := gen.next()
		if id2 != id+uint64(i)+1 {
			t.Fatalf("#%d: unexpected id %x", i, id2)
		}
	}
}

func TestGeneratorUnique(t *testing.T) {
	dur := time.Duration(100) * time.Millisecond
	gen0 := newGenerator(0, dur.Nanoseconds())
	id0 := gen0.next()

	gen1 := newGenerator(1, dur.Nanoseconds())
	id1 := gen1.next()

	if id0 == id1 {
		t.Fatalf("unexpected %x == %x", id0, id1)
	}

	dur = time.Duration(101) * time.Millisecond
	gen2 := newGenerator(0, dur.Nanoseconds())
	id2 := gen2.next()
	if id0 == id2 {
		t.Fatalf("unexpected %x == %x", id0, id2)
	}
}
