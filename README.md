# thin-sag

snapshot
curl -H "X-SAG-TOKEN: $(cat ~/Desktop/thin-sag/.thin-sag/.sagtoken)" \
     -H "Content-Type: application/json" \
     http://127.0.0.1:8900/snapshot -d '{ "window": { "index": 0 } }'