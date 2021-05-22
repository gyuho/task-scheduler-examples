package main

import (
	"math"
	"sync/atomic"
)

// https://github.com/etcd-io/etcd/blob/release-3.5/pkg/idutil
type Generator interface {
	next() uint64
}

func newGenerator(memberID uint64, unixNano int64) Generator {
	// to count the number of milliseconds
	unixMx := uint64(unixNano) / 1000000

	x := (math.MaxUint64 >> 24) & unixMx
	x = x << 8

	return &generator{
		prefix: memberID << (8 * 6),
		suffix: x,
	}
}

type generator struct {
	prefix uint64
	suffix uint64
}

func (gen *generator) next() uint64 {
	suffix := atomic.SwapUint64(&gen.suffix, gen.suffix+1)
	id := gen.prefix | (suffix & (math.MaxUint64 >> 16))
	return id
}
