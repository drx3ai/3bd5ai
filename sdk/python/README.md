# OpenAI Bd3 Python SDK

Build Python applications that start Bd3 threads, run turns, stream progress,
and control workspace access.

## Install

Install the SDK:

```bash
pip install bd3
```

## Quickstart

The SDK reuses your existing Bd3 authentication when one is already
available:

```python
from bd3 import Bd3

with Bd3() as bd3:
    thread = bd3.thread_start()
    result = thread.run("Explain this repository in three bullets.")
    print(result.final_response)
```

`thread.run(...)` returns a `TurnResult` containing the final response,
collected items, and token usage.

## Authentication

Existing Bd3 authentication is reused automatically. To start ChatGPT
browser login explicitly:

```python
from bd3 import Bd3

with Bd3() as bd3:
    login = bd3.login_chatgpt()
    print(login.auth_url)
    print(login.wait().success)
```

For device-code login:

```python
with Bd3() as bd3:
    login = bd3.login_chatgpt_device_code()
    print(login.verification_url, login.user_code)
    login.wait()
```

For API-key login:

```python
with Bd3() as bd3:
    bd3.login_api_key("sk-...")
```

## Built-In Help

Use Python's standard `help(bd3)`, `help(Bd3)`, or
`python -m pydoc bd3` documentation tools.

## Documentation

- [Getting started](https://github.com/openai/codex/blob/main/sdk/python/docs/getting-started.md)
- [API reference](https://github.com/openai/codex/blob/main/sdk/python/docs/api-reference.md)
- [FAQ](https://github.com/openai/codex/blob/main/sdk/python/docs/faq.md)
- [Examples](https://github.com/openai/codex/blob/main/sdk/python/examples/README.md)

The package is licensed under the
[repository Apache License 2.0](https://github.com/openai/codex/blob/main/LICENSE).
