access_by_lua_block {
  local redis = require "resty.redis"
  local red = redis:new()
  red:set_timeout(50)

  local ok, err = red:connect("redis", 6379)
  if not ok then
    return  -- dev: fail-open
  end

  local limit = tonumber(ngx.var.rl_limit) or 60
  local window = tonumber(ngx.var.rl_window) or 60

  -- identity: IP-based (upgrade to API key/JWT later)
  local ip = ngx.var.binary_remote_addr
  local path = ngx.var.uri or ""
  local key = "rl:ip:" .. ngx.encode_base64(ip) .. ":p:" .. path

  -- Atomic: INCR, set EXPIRE if first, return TTL, allow/deny
  local script = [[
    local current = redis.call("INCR", KEYS[1])
    if current == 1 then
      redis.call("EXPIRE", KEYS[1], ARGV[1])
    end

    local ttl = redis.call("TTL", KEYS[1])
    if ttl < 0 then
      ttl = tonumber(ARGV[1])
      redis.call("EXPIRE", KEYS[1], ttl)
    end

    if current > tonumber(ARGV[2]) then
      return {0, current, ttl}
    end
    return {1, current, ttl}
  ]]

  local res, eval_err = red:eval(script, 1, key, window, limit)
  if not res then
    return
  end

  local allowed = tonumber(res[1]) or 1
  local current = tonumber(res[2]) or 0
  local ttl = tonumber(res[3]) or window

  local remaining = limit - current
  if remaining < 0 then remaining = 0 end

  -- headers
  ngx.header["X-RateLimit-Limit"] = limit
  ngx.header["X-RateLimit-Remaining"] = remaining
  ngx.header["X-RateLimit-Reset"] = ngx.time() + ttl

  if allowed == 0 then
    ngx.status = 429
    ngx.header["Content-Type"] = "application/json"
    ngx.header["Retry-After"] = ttl
    ngx.say('{"error":"rate_limited","retry_after":' .. ttl .. '}')
    return ngx.exit(429)
  end

  red:set_keepalive(10000, 50)
}
