# thin-sag

snapshot
curl -H "X-SAG-TOKEN: $(cat ~/.thin-sag/.sagtoken)" \
     -H "Content-Type: application/json" \
     -d '{ "window": { "index": 2 } }' \
     http://127.0.0.1:8900/snapshot