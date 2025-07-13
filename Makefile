.PHONY: pre-push test-hurl clean

pre-push:
	cargo fmt --all

test-hurl: barefoot-mcp.pid
	@echo "Waiting for barefoot MCP server to be ready..."
	@timeout 20 bash -c 'until curl -s http://127.0.0.1:4196/mcp > /dev/null; do sleep 1; done'
	hurl hurl/mcp_tools.hurl --variable PORT=4196
	@echo "Killing barefoot MCP server..."
	@kill `cat barefoot-mcp.pid` || true
	@rm -f barefoot-mcp.pid

barefoot-mcp.pid:
	@echo "Starting barefoot MCP server on 127.0.0.1:4196..."
	barefoot mcp --transport http --http-host 127.0.0.1 --http-port 4196 --config examples/mcp-config.toml & echo $$! > barefoot-mcp.pid
	@sleep 2

clean:
	rm -f barefoot-mcp.pid 