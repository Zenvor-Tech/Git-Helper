#!/usr/bin/env python3
import json
import os
import ssl
import subprocess
import sys
import urllib.request
import urllib.error
from pathlib import Path


def make_url_opener(config):
    verify = config.get("SSL_VERIFY", "true").lower()
    ctx = ssl.create_default_context()
    if verify == "false":
        ctx.check_hostname = False
        ctx.verify_mode = ssl.CERT_NONE
    return urllib.request.build_opener(urllib.request.HTTPSHandler(context=ctx))


def open_url(opener, req):
    try:
        return opener.open(req)
    except urllib.error.URLError as e:
        if isinstance(e.reason, ssl.SSLCertVerificationError):
            ctx = ssl.create_default_context()
            ctx.check_hostname = False
            ctx.verify_mode = ssl.CERT_NONE
            fallback = urllib.request.build_opener(urllib.request.HTTPSHandler(context=ctx))
            return fallback.open(req)
        raise


def load_env(env_path):
    config = {}
    if not env_path.exists():
        return config
    for line in env_path.read_text().splitlines():
        line = line.strip()
        if not line or line.startswith("#") or "=" not in line:
            continue
        key, _, value = line.partition("=")
        config[key.strip().upper()] = value.strip()
    return config


def get_git_diff():
    result = subprocess.run(
        ["git", "diff", "--cached"],
        capture_output=True, text=True, cwd=os.getcwd()
    )
    staged = result.stdout.strip()

    if not staged:
        result = subprocess.run(
            ["git", "diff"],
            capture_output=True, text=True, cwd=os.getcwd()
        )
        staged = result.stdout.strip()

    if not staged:
        result = subprocess.run(
            ["git", "status", "--short"],
            capture_output=True, text=True, cwd=os.getcwd()
        )
        status_output = result.stdout.strip()
        if not status_output:
            return {"error": "no_changes"}
        staged = f"No detailed diff available. Changes:\n{status_output}"

    return {"diff": staged}


def get_git_log():
    result = subprocess.run(
        ["git", "log", "--oneline", "-10"],
        capture_output=True, text=True, cwd=os.getcwd()
    )
    return result.stdout.strip()


def get_branch_name():
    result = subprocess.run(
        ["git", "rev-parse", "--abbrev-ref", "HEAD"],
        capture_output=True, text=True, cwd=os.getcwd()
    )
    return result.stdout.strip()


def build_prompt(diff, branch, recent_log):
    return f"""You are a Git assistant that generates concise, well-formatted commit messages.

Given the following information, generate a multi-line commit message following conventional commits format.

Branch: {branch}
Recent commits on this branch:
{recent_log}

Diff / Changes:
{diff}

Rules:
1. Start with a type prefix: feat, fix, refactor, test, docs, style, chore, perf, ci, build, revert
2. First line: max 72 chars, like "feat: add user login"
3. Leave a blank line after the title
4. Body: bullet points explaining what and why, wrapped at 72 chars
5. If there's a breaking change, add "BREAKING CHANGE:" at the end

Return ONLY the commit message, no extra text, no markdown formatting."""


def call_openai_compatible(config, prompt, api_key_key, model_key, url_key, default_url, name):
    opener = make_url_opener(config)
    api_key = config.get(api_key_key)
    model = config.get(model_key, "gpt-4o")
    base_url = config.get(url_key, default_url).rstrip("/")

    data = json.dumps({
        "model": model,
        "messages": [{"role": "user", "content": prompt}],
        "max_tokens": 500,
        "temperature": 0.3,
    }).encode("utf-8")

    req = urllib.request.Request(
        f"{base_url}/chat/completions",
        data=data,
        headers={
            "Content-Type": "application/json",
            "Authorization": f"Bearer {api_key}",
        },
        method="POST",
    )

    try:
        resp = open_url(opener, req)
        body = json.loads(resp.read())
        return body["choices"][0]["message"]["content"].strip()
    except urllib.error.HTTPError as e:
        return f"error: {name} API error: {e.code} {e.read().decode()}"
    except Exception as e:
        return f"error: {name}: {e}"


def call_openai(config, prompt):
    return call_openai_compatible(config, prompt, "OPENAI_API_KEY", "OPENAI_MODEL",
                                  "OPENAI_URL", "https://api.openai.com/v1", "OpenAI")


def call_groq(config, prompt):
    return call_openai_compatible(config, prompt, "GROQ_API_KEY", "GROQ_MODEL",
                                  "GROQ_URL", "https://api.groq.com/openai/v1", "Groq")


def call_deepseek(config, prompt):
    return call_openai_compatible(config, prompt, "DEEPSEEK_API_KEY", "DEEPSEEK_MODEL",
                                  "DEEPSEEK_URL", "https://api.deepseek.com/v1", "DeepSeek")


def call_openrouter(config, prompt):
    return call_openai_compatible(config, prompt, "OPENROUTER_API_KEY", "OPENROUTER_MODEL",
                                  "OPENROUTER_URL", "https://openrouter.ai/api/v1", "OpenRouter")


def call_together(config, prompt):
    return call_openai_compatible(config, prompt, "TOGETHER_API_KEY", "TOGETHER_MODEL",
                                  "TOGETHER_URL", "https://api.together.xyz/v1", "Together")


def call_anthropic(config, prompt):
    opener = make_url_opener(config)
    api_key = config.get("ANTHROPIC_API_KEY")
    model = config.get("ANTHROPIC_MODEL", "claude-sonnet-4-20250514")

    data = json.dumps({
        "model": model,
        "max_tokens": 500,
        "messages": [{"role": "user", "content": prompt}],
    }).encode("utf-8")

    req = urllib.request.Request(
        "https://api.anthropic.com/v1/messages",
        data=data,
        headers={
            "Content-Type": "application/json",
            "x-api-key": api_key,
            "anthropic-version": "2023-06-01",
        },
        method="POST",
    )

    try:
        resp = open_url(opener, req)
        body = json.loads(resp.read())
        return body["content"][0]["text"].strip()
    except urllib.error.HTTPError as e:
        return f"error: Anthropic API error: {e.code} {e.read().decode()}"
    except Exception as e:
        return f"error: {e}"


def call_gemini(config, prompt):
    opener = make_url_opener(config)
    api_key = config.get("GEMINI_API_KEY")
    model = config.get("GEMINI_MODEL", "gemini-2.5-flash")

    data = json.dumps({
        "contents": [{"parts": [{"text": prompt}]}],
        "generationConfig": {"maxOutputTokens": 500, "temperature": 0.3},
    }).encode("utf-8")

    url = f"https://generativelanguage.googleapis.com/v1beta/models/{model}:generateContent?key={api_key}"
    req = urllib.request.Request(
        url, data=data,
        headers={"Content-Type": "application/json"},
        method="POST",
    )

    try:
        resp = open_url(opener, req)
        body = json.loads(resp.read())
        return body["candidates"][0]["content"]["parts"][0]["text"].strip()
    except urllib.error.HTTPError as e:
        return f"error: Gemini API error: {e.code} {e.read().decode()}"
    except Exception as e:
        return f"error: {e}"


def call_ollama(config, prompt):
    opener = make_url_opener(config)
    url = config.get("OLLAMA_URL", "http://localhost:11434").rstrip("/")
    model = config.get("OLLAMA_MODEL", "llama3")

    data = json.dumps({
        "model": model,
        "prompt": prompt,
        "stream": False,
        "options": {"temperature": 0.3, "num_predict": 500},
    }).encode("utf-8")

    req = urllib.request.Request(
        f"{url}/api/generate",
        data=data,
        headers={"Content-Type": "application/json"},
        method="POST",
    )

    try:
        resp = open_url(opener, req)
        body = json.loads(resp.read())
        text = body.get("response", "").strip()
        return text
    except urllib.error.HTTPError as e:
        return f"error: Ollama API error: {e.code} {e.read().decode()}"
    except Exception as e:
        return f"error: {e}"


def main():
    repo_root = subprocess.run(
        ["git", "rev-parse", "--show-toplevel"],
        capture_output=True, text=True
    ).stdout.strip()

    env_path = Path(repo_root) / ".env.local"
    config = load_env(env_path)

    provider = config.get("AI_PROVIDER", "openai").lower()

    api_key_checks = {
        "openai": ("OPENAI_API_KEY", "sk-"),
        "anthropic": ("ANTHROPIC_API_KEY", "sk-ant-"),
        "gemini": ("GEMINI_API_KEY", ""),
        "groq": ("GROQ_API_KEY", "gsk-"),
        "deepseek": ("DEEPSEEK_API_KEY", "sk-"),
        "openrouter": ("OPENROUTER_API_KEY", "sk-or-"),
        "together": ("TOGETHER_API_KEY", ""),
        "ollama": ("", ""),
    }

    if provider in api_key_checks:
        key_name, prefix = api_key_checks[provider]
        if key_name:
            api_key = config.get(key_name, "")
            if not api_key:
                print(f"error: {key_name} is not set in .env.local", file=sys.stderr)
                sys.exit(1)
            if prefix and not api_key.startswith(prefix):
                print(f"warning: {key_name} may be invalid (should start with '{prefix}...')", file=sys.stderr)

    diff_info = get_git_diff()
    if "error" in diff_info:
        if diff_info["error"] == "no_changes":
            print("No changes detected. Stage or modify files first.", file=sys.stderr)
            sys.exit(1)
        print(diff_info["error"], file=sys.stderr)
        sys.exit(1)

    diff = diff_info["diff"]
    branch = get_branch_name()
    recent_log = get_git_log()
    prompt = build_prompt(diff, branch, recent_log)

    providers = {
        "openai": call_openai,
        "anthropic": call_anthropic,
        "gemini": call_gemini,
        "groq": call_groq,
        "deepseek": call_deepseek,
        "openrouter": call_openrouter,
        "together": call_together,
        "ollama": call_ollama,
    }

    if provider not in providers:
        print(f"error: unknown provider '{provider}'. Choose from: {', '.join(providers.keys())}", file=sys.stderr)
        sys.exit(1)

    result = providers[provider](config, prompt)

    if result.startswith("error:"):
        print(result, file=sys.stderr)
        sys.exit(1)

    print(result)


if __name__ == "__main__":
    main()
