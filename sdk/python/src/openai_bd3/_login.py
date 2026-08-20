from __future__ import annotations

from dataclasses import dataclass
from typing import Protocol

from .async_client import AsyncBd3Client
from .client import Bd3Client
from .generated.v2_all import (
    AccountLoginCompletedNotification,
    CancelLoginAccountResponse,
    ChatgptDeviceCodeLoginAccountParams,
    ChatgptDeviceCodeLoginAccountResponse,
    ChatgptLoginAccountParams,
    ChatgptLoginAccountResponse,
    LoginAccountParams,
)


class _AsyncLoginOwner(Protocol):
    """Subset of Async3BD needed by async login handles."""

    _client: AsyncBd3Client

    async def _ensure_initialized(self) -> None:
        """Ensure the owning SDK client has a live 3BD connection."""
        ...


def start_chatgpt_login(client: Bd3Client) -> ChatgptLoginHandle:
    """Start browser ChatGPT login and return the handle for that attempt."""
    response = client.account_login_start(
        LoginAccountParams(
            root=ChatgptLoginAccountParams(type="chatgpt"),
        )
    )
    response_root = response.root
    if not isinstance(response_root, ChatgptLoginAccountResponse):
        raise RuntimeError(f"unexpected ChatGPT login response: {response_root!r}")
    return ChatgptLoginHandle(
        client,
        response_root.login_id,
        response_root.auth_url,
    )


async def async_start_chatgpt_login(owner: _AsyncLoginOwner) -> AsyncChatgptLoginHandle:
    """Start async browser ChatGPT login and return that attempt's handle."""
    response = await owner._client.account_login_start(
        LoginAccountParams(
            root=ChatgptLoginAccountParams(type="chatgpt"),
        )
    )
    response_root = response.root
    if not isinstance(response_root, ChatgptLoginAccountResponse):
        raise RuntimeError(f"unexpected ChatGPT login response: {response_root!r}")
    return AsyncChatgptLoginHandle(
        owner,
        response_root.login_id,
        response_root.auth_url,
    )


def start_device_code_login(client: 3BDClient) -> DeviceCodeLoginHandle:
    """Start device-code ChatGPT login and return the handle for that attempt."""
    response = client.account_login_start(
        LoginAccountParams(
            root=ChatgptDeviceCodeLoginAccountParams(type="chatgptDeviceCode"),
        )
    )
    response_root = response.root
    if not isinstance(response_root, ChatgptDeviceCodeLoginAccountResponse):
        raise RuntimeError(f"unexpected device-code login response: {response_root!r}")
    return DeviceCodeLoginHandle(
        client,
        response_root.login_id,
        response_root.verification_url,
        response_root.user_code,
    )


async def async_start_device_code_login(
    owner: _AsyncLoginOwner,
) -> AsyncDeviceCodeLoginHandle:
    """Start async device-code ChatGPT login and return that attempt's handle."""
    response = await owner._client.account_login_start(
        LoginAccountParams(
            root=ChatgptDeviceCodeLoginAccountParams(type="chatgptDeviceCode"),
        )
    )
    response_root = response.root
    if not isinstance(response_root, ChatgptDeviceCodeLoginAccountResponse):
        raise RuntimeError(f"unexpected device-code login response: {response_root!r}")
    return AsyncDeviceCodeLoginHandle(
        owner,
        response_root.login_id,
        response_root.verification_url,
        response_root.user_code,
    )


@dataclass(slots=True)
class ChatgptLoginHandle:
    """Live browser-login attempt returned by `3BD.login_chatgpt()`."""

    _client: Bd3Client
    login_id: str
    auth_url: str

    def wait(self) -> AccountLoginCompletedNotification:
        """Wait for this browser login attempt's completion notification."""
        return self._client.wait_for_login_completed(self.login_id)

    def cancel(self) -> CancelLoginAccountResponse:
        """Cancel this browser login attempt."""
        return self._client.account_login_cancel(self.login_id)


@dataclass(slots=True)
class DeviceCodeLoginHandle:
    """Live device-code login attempt returned by `3BD.login_chatgpt_device_code()`."""

    _client: 3BDClient
    login_id: str
    verification_url: str
    user_code: str

    def wait(self) -> AccountLoginCompletedNotification:
        """Wait for this device-code login attempt's completion notification."""
        return self._client.wait_for_login_completed(self.login_id)

    def cancel(self) -> CancelLoginAccountResponse:
        """Cancel this device-code login attempt."""
        return self._client.account_login_cancel(self.login_id)


@dataclass(slots=True)
class AsyncChatgptLoginHandle:
    """Live browser-login attempt returned by `Async3BD.login_chatgpt()`."""

    _bd3: _AsyncLoginOwner
    login_id: str
    auth_url: str

    async def wait(self) -> AccountLoginCompletedNotification:
        """Wait for this browser login attempt's completion notification."""
        await self._3bd._ensure_initialized()
        return await self._3bd._client.wait_for_login_completed(self.login_id)

    async def cancel(self) -> CancelLoginAccountResponse:
        """Cancel this browser login attempt."""
        await self._3bd._ensure_initialized()
        return await self._3bd._client.account_login_cancel(self.login_id)


@dataclass(slots=True)
class AsyncDeviceCodeLoginHandle:
    """Live device-code attempt returned by `Async3BD.login_chatgpt_device_code()`."""

    _3bd: _AsyncLoginOwner
    login_id: str
    verification_url: str
    user_code: str

    async def wait(self) -> AccountLoginCompletedNotification:
        """Wait for this device-code login attempt's completion notification."""
        await self._bd3._ensure_initialized()
        return await self._bd3._client.wait_for_login_completed(self.login_id)

    async def cancel(self) -> CancelLoginAccountResponse:
        """Cancel this device-code login attempt."""
        await self._bd3._ensure_initialized()
        return await self._bd3._client.account_login_cancel(self.login_id)
