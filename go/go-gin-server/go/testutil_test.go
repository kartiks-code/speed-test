package petstore

import (
	"bytes"
	"encoding/json"
	"net/http/httptest"

	"github.com/gin-gonic/gin"
)

// testRequest describes a single handler invocation. It is executed against a
// fresh Gin test context (no router) so handlers can be exercised in isolation.
type testRequest struct {
	method  string
	target  string
	body    *bytes.Reader
	params  map[string]string
	handler gin.HandlerFunc
}

func (r testRequest) run() *httptest.ResponseRecorder {
	req := httptest.NewRequest(r.method, r.target, r.body)
	req.Header.Set("Content-Type", "application/json")
	rec := httptest.NewRecorder()
	ctx, _ := gin.CreateTestContext(rec)
	ctx.Request = req
	for key, value := range r.params {
		ctx.Params = append(ctx.Params, gin.Param{Key: key, Value: value})
	}
	r.handler(ctx)
	return rec
}

func jsonRequest(method, target string, body any, params map[string]string, handler gin.HandlerFunc) testRequest {
	payload, _ := json.Marshal(body)
	return testRequest{method: method, target: target, body: bytes.NewReader(payload), params: params, handler: handler}
}

func rawRequest(method, target, body string, params map[string]string, handler gin.HandlerFunc) testRequest {
	return testRequest{method: method, target: target, body: bytes.NewReader([]byte(body)), params: params, handler: handler}
}

func paramRequest(method, target string, params map[string]string, handler gin.HandlerFunc) testRequest {
	return testRequest{method: method, target: target, body: bytes.NewReader(nil), params: params, handler: handler}
}
