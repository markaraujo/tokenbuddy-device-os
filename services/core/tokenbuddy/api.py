import asyncio
import hmac
from contextlib import asynccontextmanager
from typing import Annotated, Literal

from fastapi import FastAPI, Header, HTTPException, Depends
from pydantic import BaseModel, ConfigDict, Field

from .bridge import CommandError


class Message(BaseModel):
    model_config = ConfigDict(extra="forbid", strict=True)


class AgentEvent(Message):
    event: Literal["agent.started", "agent.completed", "agent.failed"]
    agent: str = Field(min_length=1, max_length=31, pattern=r"^[ -~]+$")
    input_tokens: int | None = Field(default=None, ge=0, le=2147483647)
    output_tokens: int | None = Field(default=None, ge=0, le=2147483647)
    task_id: str | None = Field(default=None, min_length=1, max_length=64, pattern=r"^[A-Za-z0-9_-]+$")


class Settings(Message):
    brightness: int | None = Field(default=None, ge=0, le=50)
    animation: int | None = Field(default=None, ge=0, le=2)
    demo: bool | None = None


class Navigate(Message):
    page: int = Field(ge=0, le=4)


STATES = {"idle": 0, "happy": 1, "thinking": 2, "error": 3, "curious": 4,
          "working": 5, "success": 6, "sleeping": 7, "disconnected": 8, "dizzy": 9}


class Expression(Message):
    state: Literal["idle", "happy", "thinking", "error", "curious", "working", "success", "sleeping", "disconnected", "dizzy"]
    duration: int = Field(default=3000, ge=50, le=60000)


def create_app(bridge, key: str, manage_bridge=True):
    @asynccontextmanager
    async def lifespan(_app):
        if manage_bridge:
            bridge.start()
        yield
        if manage_bridge:
            await asyncio.to_thread(bridge.stop)

    app = FastAPI(title="TokenBuddy Local Host", version="0.3.0", lifespan=lifespan)

    async def authorized(x_tokenbuddy_key: Annotated[str | None, Header()] = None):
        if not x_tokenbuddy_key or not hmac.compare_digest(x_tokenbuddy_key, key):
            raise HTTPException(401, "Chave local obrigatoria")

    async def send(message):
        try:
            return await asyncio.to_thread(bridge.request, message)
        except CommandError as error:
            raise HTTPException(409, str(error)) from error
        except (ConnectionError, TimeoutError, OSError) as error:
            raise HTTPException(503, str(error)) from error

    @app.get("/health")
    async def health():
        return {"status": "ok", "version": "0.3.0"}

    @app.get("/api/status", dependencies=[Depends(authorized)])
    async def status():
        return bridge.snapshot()

    @app.post("/api/event", dependencies=[Depends(authorized)])
    async def event(value: AgentEvent):
        if (value.input_tokens is None) != (value.output_tokens is None):
            raise HTTPException(422, "Entrada e saida devem ser fornecidas juntas")
        return await send({"type": "event", **value.model_dump(exclude_none=True)})

    @app.post("/api/settings", dependencies=[Depends(authorized)])
    async def settings(value: Settings):
        return await send({"type": "settings", **value.model_dump(exclude_none=True)})

    @app.post("/api/navigate", dependencies=[Depends(authorized)])
    async def navigate(value: Navigate):
        return await send({"type": "navigate", **value.model_dump()})

    @app.post("/api/expression", dependencies=[Depends(authorized)])
    async def expression(value: Expression):
        return await send({"type": "expression", "state": STATES[value.state], "duration": value.duration})

    return app
