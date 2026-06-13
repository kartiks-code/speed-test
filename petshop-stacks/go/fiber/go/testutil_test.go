package petstore

import (
	"bytes"
	"encoding/json"
	"io"
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"

	"github.com/gofiber/fiber/v2"
)

// testRequest describes a single handler invocation. It is executed against a
// minimal Fiber app so that path params are populated correctly.
type testRequest struct {
	method  string
	target  string
	body    *bytes.Reader
	params  map[string]string // param name -> literal value in target path
	handler fiber.Handler
}

func (r testRequest) run() *http.Response {
	app := fiber.New(fiber.Config{DisableStartupMessage: true})
	pattern := buildRoutePattern(r.target, r.params)
	app.Add(r.method, pattern, r.handler)

	var body io.Reader
	if r.body != nil {
		body = r.body
	}
	req := httptest.NewRequest(r.method, r.target, body)
	req.Header.Set("Content-Type", "application/json")
	resp, err := app.Test(req, -1)
	if err != nil {
		panic("testRequest.run: " + err.Error())
	}
	return resp
}

// buildRoutePattern converts a concrete path like /api/v3/pet/1 plus params
// {"petId":"1"} into the Fiber route pattern /api/v3/pet/:petId by replacing
// path segments that exactly equal a param value with /:key.
func buildRoutePattern(target string, params map[string]string) string {
	path := target
	if i := strings.IndexByte(path, '?'); i >= 0 {
		path = path[:i]
	}
	if len(params) == 0 {
		return path
	}
	segments := strings.Split(path, "/")
	for i, seg := range segments {
		for key, value := range params {
			if seg == value {
				segments[i] = ":" + key
				break
			}
		}
	}
	return strings.Join(segments, "/")
}

func jsonRequest(method, target string, body any, params map[string]string, handler fiber.Handler) testRequest {
	payload, _ := json.Marshal(body)
	return testRequest{method: method, target: target, body: bytes.NewReader(payload), params: params, handler: handler}
}

func rawRequest(method, target, body string, params map[string]string, handler fiber.Handler) testRequest {
	return testRequest{method: method, target: target, body: bytes.NewReader([]byte(body)), params: params, handler: handler}
}

func paramRequest(method, target string, params map[string]string, handler fiber.Handler) testRequest {
	return testRequest{method: method, target: target, body: bytes.NewReader(nil), params: params, handler: handler}
}

// testApp builds a full Fiber app with all routes wired to the given store.
func testApp(t *testing.T, store Store) *fiber.App {
	t.Helper()
	return NewFiberApp(ApiHandleFunctions{
		PetAPI:   PetAPI{Store: store},
		StoreAPI: StoreAPI{Store: store},
		UserAPI:  UserAPI{Store: store},
	})
}

// doRequest sends a JSON request through a full Fiber app.
func doRequest(t *testing.T, app *fiber.App, method, path string, body interface{}) *http.Response {
	t.Helper()
	var reqBody io.Reader
	if body != nil {
		b, err := json.Marshal(body)
		if err != nil {
			t.Fatalf("marshal body: %v", err)
		}
		reqBody = bytes.NewReader(b)
	}
	req := httptest.NewRequest(method, path, reqBody)
	if body != nil {
		req.Header.Set("Content-Type", "application/json")
	}
	resp, err := app.Test(req, -1)
	if err != nil {
		t.Fatalf("app.Test: %v", err)
	}
	return resp
}

// doRawRequest sends a raw-body request through a full Fiber app.
func doRawRequest(t *testing.T, app *fiber.App, method, path, rawBody string) *http.Response {
	t.Helper()
	req := httptest.NewRequest(method, path, bytes.NewReader([]byte(rawBody)))
	req.Header.Set("Content-Type", "application/json")
	resp, err := app.Test(req, -1)
	if err != nil {
		t.Fatalf("app.Test: %v", err)
	}
	return resp
}

func decodeBody(t *testing.T, resp *http.Response, target any) {
	t.Helper()
	defer resp.Body.Close()
	data, err := io.ReadAll(resp.Body)
	if err != nil {
		t.Fatalf("read body: %v", err)
	}
	if err := json.Unmarshal(data, target); err != nil {
		t.Fatalf("decode response %q: %v", string(data), err)
	}
}

// newTestApp is a helper that creates a Fiber app with a single route for
// testing context-dependent helper functions.
func newTestApp(method, path string, handler fiber.Handler) *fiber.App {
	app := fiber.New(fiber.Config{DisableStartupMessage: true})
	app.Add(method, path, handler)
	return app
}

// sendTo sends a request to a single-handler test app and returns the response.
func sendTo(app *fiber.App, method, path, body string) *http.Response {
	req := httptest.NewRequest(method, path, strings.NewReader(body))
	req.Header.Set("Content-Type", "application/json")
	resp, err := app.Test(req, -1)
	if err != nil {
		panic("sendTo: " + err.Error())
	}
	return resp
}
