package main

import (
	"encoding/json"
	"fmt"
	"net/http"
	"os"
	"os/signal"
	"syscall"
	"time"
)

type Handler interface {
	start()
}

type handler struct {
	listenerPort uint64
	applier      Applier
}

func newHandler(listenerPort uint64, requestTimeout time.Duration) Handler {
	return &handler{
		listenerPort: listenerPort,
		applier:      newApplier(requestTimeout),
	}
}

func (hd *handler) start() {
	fmt.Println("starting server")
	hd.applier.start()

	// start listener
	serverMux := http.NewServeMux()
	serverMux.HandleFunc("/echo", hd.wrapFunc(handleRequest))

	httpServer := &http.Server{
		Addr:    fmt.Sprintf(":%d", hd.listenerPort),
		Handler: serverMux,
	}

	tch := make(chan os.Signal, 1)
	signal.Notify(tch, syscall.SIGINT)
	done := make(chan struct{})
	go func() {
		fmt.Println("received signal:", <-tch)
		httpServer.Close()
		close(done)
	}()

	fmt.Printf("Serving http://localhost:%d\n", hd.listenerPort)
	if err := httpServer.ListenAndServe(); err != nil {
		fmt.Printf("http server error: %v\n", err)
	}
	select {
	case <-done:
	default:
	}

	if err := hd.applier.stop(); err != nil {
		fmt.Printf("failed to stop applier %v", err)
		panic(err)
	}
}

func (hd *handler) wrapFunc(fn func(applier Applier, w http.ResponseWriter, req *http.Request)) func(w http.ResponseWriter, req *http.Request) {
	return func(w http.ResponseWriter, req *http.Request) {
		fn(hd.applier, w, req)
	}
}

func handleRequest(applier Applier, w http.ResponseWriter, req *http.Request) {
	switch req.Method {
	case "POST":
		var echoRequest EchoRequest
		err := json.NewDecoder(req.Body).Decode(&echoRequest)
		if err != nil {
			fmt.Fprintf(w, "failed to read request %v", err)
			return
		}
		s, err := applier.apply(Request{echoRequest: &echoRequest})
		if err != nil {
			fmt.Fprintf(w, "failed to apply request %v", err)
			return
		}
		fmt.Fprint(w, s)

	default:
		http.Error(w, "Method Not Allowed", 405)
	}
}
