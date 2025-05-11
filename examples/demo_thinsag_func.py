import os, json, requests, openai

OPENAI_API_KEY = os.environ["OPENAI_API_KEY"]
SAG_HOST        = "http://127.0.0.1:8900"

openai.api_key = OPENAI_API_KEY
prompt = """You are an assistant that controls my Mac via Thin‑SAG.
Goal: write “Hello {secret.profile_name}” into Notes and save.
Notes' bundle name is com.apple.Notes.

Use {secret.profile_name}.
"""
functions = [
    {
        "name": "run_actions",
        "description": "Send a sequence of UI actions to Thin‑SAG",
        "parameters": {
            "type": "object",
            "properties": {
                "actions": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "required": ["act"],
                        "properties": {
                            "act":   {"type": "string", "enum": ["launch", "type", "click", "wait"]},
                            "target": {"type": "string"},
                            "text":   {"type": "string"},
                            "x":      {"type": "integer"},
                            "y":      {"type": "integer"},
                            "ms":     {"type": "integer"}
                        }
                    }
                }
            },
            "required": ["actions"]
        }
    }
]



resp = openai.ChatCompletion.create(
    model="gpt-4o-mini",
    messages=[{"role":"user","content":prompt}],
    functions=functions,
    function_call={"name":"run_actions"}
)
actions = json.loads(resp.choices[0].message.function_call.arguments)

print("LLM output actions ➜", json.dumps(actions, indent=2))


r = requests.post(f"{SAG_HOST}/run-json",
                  headers={"Content-Type": "application/json"},
                  data=json.dumps(actions["actions"]))  # 修正: actions["actions"] を送信

if r.status_code != 200:
    print(f"Error: Received status code {r.status_code}")
    print("Response text:", r.text)
    exit(1)

job_id = r.json()["job_id"]
print("Job ID =", job_id)

# ③ ポーリングして結果取得
while True:
    status = requests.get(f"{SAG_HOST}/job/{job_id}").json()
    if status["status"] != "Running":
        break

print("Final Status =", status)
