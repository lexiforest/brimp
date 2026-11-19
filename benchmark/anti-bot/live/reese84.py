"""`pyv8c/examples/reese84/reese84.py`, ported to Brimp.

Same origin, same SDK path, same API call; the cookie wait reads the session jar
instead of a profile, and there is no resource interceptor (Brimp fetches what the
page asks for).
"""
from __future__ import annotations

import json
import time

import brimp

ORIGIN = "https://orion.nv.gov"
URL = ORIGIN + "/portal/public/#/public/nvsos/en/CaseXscreen?screen=external-GenericFilingsSearch&tabRoute=agent"
API_URL = ORIGIN + "/core/api/_q/v1/nv-sos/public/CRAEntities"
KEYWORD = "NORTHWEST"


def header_for(cookies, name):
    values = [record["value"] for record in cookies if record["name"] == name and record["value"]]
    return values[0] if values else None


def search_body(keyword, page_number=1, page_size=25):
    params = {
        "agentName": keyword, "registeredAgentType": "", "craId": "", "nvBusinessId": "",
        "craEntityNumber": "", "agentStatus": "", "filingNo": "", "submit1": False,
        "searchDumy": True, "restrictedAndProhibitedWords": "", "totalRecordCount": "",
        "searchType": "STARTS_WITH", "searchTerm": "",
    }
    return {
        "data": {}, "origin": "", "origin_key": "CaseX",
        "gridInput": {"params": {"parameters": params.copy()}, "persistPrevParams": False},
        "parameters": params, "pageNo": page_number, "pageSize": page_size, "sortBy": {},
    }


def main() -> int:
    started = time.monotonic()
    with brimp.Session() as session:
        page = session.get(URL, timeout=60)
        print("document:", page.status_code, page.url, f"{time.monotonic() - started:.1f}s")
        deadline = started + 45
        token = None
        while time.monotonic() < deadline:
            page.run_pending_tasks()
            time.sleep(0.25)
            token = header_for(session.cookies, "reese84")
            if token:
                break
        print("reese84:", (token or "")[:40], f"{time.monotonic() - started:.1f}s")
        if not token:
            print("cookies:", sorted(record["name"] for record in session.cookies))
            return 1
        response = session.post(API_URL, json=search_body(KEYWORD), timeout=30)
        print("api:", response.status_code, f"{time.monotonic() - started:.1f}s")
        try:
            body = response.json()
        except Exception:  # noqa: BLE001
            print("body:", response.text[:200])
            return 1
        print("code:", body.get("code"), "total:", body.get("totalItemCount"))
        agents = [item.get("agentname") for item in body.get("items", [])][:5]
        print("agents:", json.dumps(agents, ensure_ascii=False))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
