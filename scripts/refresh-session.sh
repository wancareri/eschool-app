#!/usr/bin/env bash
# e-school session manager — login + auto-refresh via bash/curl.
#
# Usage:
#   bash scripts/refresh-session.sh --login Kuskov11 28012010Yh!
#   bash scripts/refresh-session.sh
#   bash scripts/refresh-session.sh --status
#   bash scripts/refresh-session.sh --loop

set -euo pipefail

BASE="https://diary.e-schools.by"
OAUTH="https://oauth.rios.unibel.by"
TOKEN_FILE="$HOME/.eschool-tokens.json"
COOKIE_JAR="/tmp/eschool-cookies.txt"
UA="Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.0 Mobile/15E148 Safari/604.1"

CURL="curl -sS -k -c $COOKIE_JAR -b $COOKIE_JAR --max-time 15"

# ── extract header value from file ──────────────────────────────────────
hdr() { grep -i "^$1:" "$2" 2>/dev/null | tail -1 | sed "s/^$1: *//i" | tr -d '\r'; }

# ── login ───────────────────────────────────────────────────────────────
do_login() {
  local user="$1" pass="$2"
  rm -f "$COOKIE_JAR"

  echo "[1/7] GET diary login/student..."
  $CURL -H "User-Agent: $UA" -D /tmp/s1.txt "$BASE/api/v1/admin/auth/login/student" -o /dev/null
  local loc1
  loc1=$(hdr location /tmp/s1.txt)
  [[ -z "$loc1" ]] && { echo "FAIL step 1"; cat /tmp/s1.txt; exit 1; }
  echo "  → ${loc1:0:100}..."

  echo "[2/7] GET OAuth login page..."
  local login_url="${loc1}"
  [[ "$login_url" != http* ]] && login_url="$OAUTH$loc1"
  $CURL -H "User-Agent: $UA" "$login_url" -o /tmp/s2.html
  local csrf return_url origin
  csrf=$(grep -oP 'name="__RequestVerificationToken"[^>]*value="\K[^"]+' /tmp/s2.html)
  return_url=$(grep -oP 'name="Input\.ReturnUrl"\s+value="\K[^"]+' /tmp/s2.html | sed 's/&amp;/\&/g')
  origin=$(echo "$login_url" | grep -oP '^https?://[^/]+')
  [[ -z "$csrf" ]] && { echo "FAIL: no CSRF"; exit 1; }
  [[ -z "$return_url" ]] && { echo "FAIL: no ReturnUrl"; exit 1; }
  echo "  CSRF=${csrf:0:20}... origin=$origin"

  echo "[3/7] POST credentials..."
  $CURL -H "User-Agent: $UA" -H "Content-Type: application/x-www-form-urlencoded" \
    -D /tmp/s3.txt -X POST \
    --data-urlencode "Input.ReturnUrl=$return_url" \
    --data-urlencode "Input.Username=$user" \
    --data-urlencode "Input.Password=$pass" \
    --data-urlencode "Input.Button=login" \
    --data-urlencode "Input.RememberLogin=false" \
    --data-urlencode "__RequestVerificationToken=$csrf" \
    "$origin/Account/Login" -o /dev/null
  local loc3
  loc3=$(hdr location /tmp/s3.txt)
  [[ -z "$loc3" ]] && { echo "FAIL step 3 — no Location"; cat /tmp/s3.txt; exit 1; }
  echo "  → ${loc3:0:100}..."

  echo "[4/7] GET callback..."
  local cb_url="$loc3"
  [[ "$cb_url" != http* ]] && cb_url="$origin$loc3"
  # URL-encode spaces in the callback URL
  cb_url=$(python3 -c "import urllib.parse; print(urllib.parse.quote('$cb_url', safe=':/?#[]@!$&\'()*+,;=-._~%'))")
  $CURL -H "User-Agent: $UA" -H "Referer: $origin/" -D /tmp/s4.txt "$cb_url" -o /dev/null
  local loc4
  loc4=$(hdr location /tmp/s4.txt)
  [[ -z "$loc4" ]] && { echo "FAIL step 4 — no Location"; cat /tmp/s4.txt; exit 1; }
  echo "  → ${loc4:0:100}..."

  echo "[5/7] GET diary callback..."
  local diag_url="$loc4"
  [[ "$diag_url" != http* ]] && diag_url="$BASE$loc4"
  $CURL -H "User-Agent: $UA" -D /tmp/s5.txt "$diag_url" -o /dev/null
  local loc5
  loc5=$(hdr location /tmp/s5.txt)
  [[ -z "$loc5" ]] && { echo "FAIL step 5 — no Location"; cat /tmp/s5.txt; exit 1; }
  echo "  → ${loc5:0:100}..."
  local uuid
  uuid=$(echo "$loc5" | grep -oP 'preauthorized\?data=\K[^&]+')
  [[ -z "$uuid" ]] && { echo "FAIL: no UUID in $loc5"; exit 1; }

  echo "[6/7] GET data_for_login/$uuid..."
  $CURL -H "User-Agent: $UA" "$BASE/api/v1/admin/auth/data_for_login/$uuid" -o /tmp/s6.json
  local profile_id school_id kind
  profile_id=$(python3 -c "import json; d=json.load(open('/tmp/s6.json')); print(d['profile_id'])")
  school_id=$(python3 -c "import json; d=json.load(open('/tmp/s6.json')); print(d['schools'][0]['id'])")
  kind=$(python3 -c "import json; d=json.load(open('/tmp/s6.json')); print(d.get('kinds',['student'])[0].lower())")
  echo "  profile=$profile_id school=$school_id kind=$kind"

  echo "[7/7] GET auth/login..."
  local token_payload="${profile_id}:${school_id}:${kind}"
  local token_b64
  token_b64=$(python3 -c "import base64,urllib.parse; print(urllib.parse.quote(base64.b64encode(b'$token_payload').decode()))")
  $CURL -H "User-Agent: $UA" "$BASE/api/v1/auth/login?token=$token_b64" -o /tmp/s7.json
  local access refresh
  access=$(python3 -c "import json; print(json.load(open('/tmp/s7.json'))['auth_token'])")
  refresh=$(python3 -c "import json; print(json.load(open('/tmp/s7.json'))['refresh_token'])")

  python3 -c "
import json, time
t={'access_token':'$access','refresh_token':'$refresh','saved_at':int(time.time()*1000)}
open('$TOKEN_FILE','w').write(json.dumps(t,indent=2))
"
  echo "✓ Login OK. Token saved to $TOKEN_FILE"
  rm -f "$COOKIE_JAR" /tmp/s*.txt /tmp/s*.html /tmp/s*.json
}

# ── refresh ─────────────────────────────────────────────────────────────
do_refresh() {
  [[ ! -f "$TOKEN_FILE" ]] && { echo "No tokens. Run --login first."; exit 1; }
  local refresh
  refresh=$(python3 -c "import json; print(json.load(open('$TOKEN_FILE'))['refresh_token'])")
  local access
  access=$(python3 -c "import json; print(json.load(open('$TOKEN_FILE')).get('access_token',''))")

  # Check if still valid
  if [[ -n "$access" ]]; then
    local exp
    exp=$(python3 -c "
import json,base64
try:
  p=json.loads(base64.urlsafe_b64decode(access.split('.')[1].split('.')[0]+'=='))
  print(p.get('exp',0))
except: print(0)
" 2>/dev/null || echo 0)
    local now
    now=$(python3 -c "import time; print(int(time.time()))")
    local left=$(( exp - now ))
    if [[ $left -gt 300 ]]; then
      echo "Token valid for $((left/60))m. Skipping."
      return
    fi
  fi

  echo "Refreshing..."
  local resp
  resp=$(curl -sS -k --max-time 15 -X POST \
    -H "Content-Type: application/x-www-form-urlencoded" \
    -H "User-Agent: $UA" \
    -d "client_id=oauth_diary_echools" \
    -d "grant_type=refresh_token" \
    -d "refresh_token=$refresh" \
    "$OAUTH/connect/token")

  if echo "$resp" | grep -q "auth_token"; then
    local new_access new_refresh
    new_access=$(echo "$resp" | python3 -c "import json,sys; print(json.load(sys.stdin)['auth_token'])")
    new_refresh=$(echo "$resp" | python3 -c "import json,sys; d=json.load(sys.stdin); print(d.get('refresh_token','$refresh'))")
    python3 -c "
import json,time
t={'access_token':'$new_access','refresh_token':'$new_refresh','saved_at':int(time.time()*1000)}
open('$TOKEN_FILE','w').write(json.dumps(t,indent=2))
"
    echo "✓ Refreshed."
  else
    echo "✗ Refresh failed (will re-login next time)"
    # Token is still valid, just can't refresh yet — keep current tokens
    return 1
  fi
}

# ── status ──────────────────────────────────────────────────────────────
do_status() {
  [[ ! -f "$TOKEN_FILE" ]] && { echo "No tokens. Run --login first."; exit 1; }
  python3 -c "
import json,base64,time
t=json.load(open('$TOKEN_FILE'))
try:
  p=json.loads(base64.urlsafe_b64decode(t['access_token'].split('.')[1].split('.')[0]+'=='))
  exp=p['exp']
  now=int(time.time())
  left=exp-now
  if left>0: print(f'Token valid. Expires in {left//60}m {left%60}s')
  else: print(f'Token EXPIRED {abs(left)//60}m ago')
except: print('Cannot decode token')
print(f'Saved: {time.strftime(\"%Y-%m-%d %H:%M:%S\", time.localtime(t[\"saved_at\"]/1000))}')
"
}

# ── entry ───────────────────────────────────────────────────────────────
case "${1:-}" in
  --login)
    do_login "$2" "$3"
    ;;
  --status)
    do_status
    ;;
  --loop)
    echo "Auto-refresh loop (30 min). Ctrl+C to stop."
    do_refresh
    while true; do sleep 1800; do_refresh; done
    ;;
  *)
    do_refresh
    ;;
esac
