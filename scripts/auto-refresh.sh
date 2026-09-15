#!/bin/bash
# Auto re-login when token is about to expire (< 10 min left).
# Uses PUT /api/v1/auth/refresh (the web app's actual refresh endpoint).

TOKEN_FILE="$HOME/.eschool-tokens.json"
BASE="https://diary.e-schools.by"

# Check if token exists and time left
if [ ! -f "$TOKEN_FILE" ]; then
    exit 1
fi

python3 -c "
import json,base64,time,sys
t=json.load(open('$TOKEN_FILE'))
p=json.loads(base64.urlsafe_b64decode(t['access_token'].split('.')[1]+'=='))
left=p['exp']-int(time.time())
if left > 600:
    sys.exit(0)
sys.exit(2)
" 2>/dev/null
STATUS=$?

if [ $STATUS -eq 0 ]; then
    exit 0  # Token still valid
fi

# Token expiring — try refresh
REFRESH=$(python3 -c "import json; print(json.load(open('$TOKEN_FILE'))['refresh_token'])")

RESULT=$(curl -sS -k --max-time 15 \
    -X PUT \
    -H "Content-Type: text/plain" \
    -d "$REFRESH" \
    "$BASE/api/v1/auth/refresh" 2>/dev/null)

AUTH_TOKEN=$(echo "$RESULT" | python3 -c "import sys,json; print(json.load(sys.stdin)['auth_token'])" 2>/dev/null)
NEW_REFRESH=$(echo "$RESULT" | python3 -c "import sys,json; print(json.load(sys.stdin)['refresh_token'])" 2>/dev/null)

if [ -n "$AUTH_TOKEN" ] && [ -n "$NEW_REFRESH" ]; then
    python3 -c "
import json,time
t={'access_token':'$AUTH_TOKEN','refresh_token':'$NEW_REFRESH','saved_at':int(time.time()*1000)}
json.dump(t,open('$TOKEN_FILE','w'),indent=2)
print('Refresh OK')
"
else
    echo "Refresh failed, trying full login..."
    # Could add full login fallback here
fi
