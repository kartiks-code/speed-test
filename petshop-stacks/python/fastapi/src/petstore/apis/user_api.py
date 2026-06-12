# coding: utf-8

from typing import Dict, List  # noqa: F401
import importlib
import pkgutil

from petstore.apis.user_api_base import BaseUserApi
import petstore.petstore.impl

from fastapi import (  # noqa: F401
    APIRouter,
    Body,
    Cookie,
    Depends,
    Form,
    Header,
    HTTPException,
    Path,
    Query,
    Response,
    Security,
    status,
)

from petstore.models.extra_models import TokenModel  # noqa: F401
from pydantic import Field, StrictStr
from typing import Any, List, Optional
from typing_extensions import Annotated
from petstore.models.error import Error
from petstore.models.user import User


router = APIRouter()

ns_pkg = petstore.petstore.impl
for _, name, _ in pkgutil.iter_modules(ns_pkg.__path__, ns_pkg.__name__ + "."):
    importlib.import_module(name)


@router.post(
    "/user",
    responses={
        200: {"model": User, "description": "successful operation"},
        "default": {"model": Error, "description": "Unexpected error"},
    },
    tags=["user"],
    summary="Create user.",
    response_model_by_alias=True,
)
async def create_user(
    user: Annotated[Optional[User], Field(description="Created user object")] = Body(None, description="Created user object"),
) -> User:
    """This can only be done by the logged in user."""
    if not BaseUserApi.subclasses:
        raise HTTPException(status_code=500, detail="Not implemented")
    return await BaseUserApi.subclasses[0]().create_user(user)


@router.post(
    "/user/createWithList",
    responses={
        200: {"model": User, "description": "Successful operation"},
        "default": {"model": Error, "description": "Unexpected error"},
    },
    tags=["user"],
    summary="Creates list of users with given input array.",
    response_model_by_alias=True,
)
async def create_users_with_list_input(
    user: Optional[List[User]] = Body(None, description=""),
) -> User:
    """Creates list of users with given input array."""
    if not BaseUserApi.subclasses:
        raise HTTPException(status_code=500, detail="Not implemented")
    return await BaseUserApi.subclasses[0]().create_users_with_list_input(user)


@router.get(
    "/user/login",
    responses={
        200: {"model": str, "description": "successful operation"},
        400: {"description": "Invalid username/password supplied"},
        "default": {"model": Error, "description": "Unexpected error"},
    },
    tags=["user"],
    summary="Logs user into the system.",
    response_model_by_alias=True,
)
async def login_user(
    username: Annotated[Optional[StrictStr], Field(description="The user name for login")] = Query(None, description="The user name for login", alias="username"),
    password: Annotated[Optional[StrictStr], Field(description="The password for login in clear text")] = Query(None, description="The password for login in clear text", alias="password"),
) -> str:
    """log user into the system."""
    if not BaseUserApi.subclasses:
        raise HTTPException(status_code=500, detail="Not implemented")
    return await BaseUserApi.subclasses[0]().login_user(username, password)


@router.get(
    "/user/logout",
    responses={
        200: {"description": "successful operation"},
        "default": {"description": "successful operation"},
    },
    tags=["user"],
    summary="Logs out current logged in user session.",
    response_model_by_alias=True,
)
async def logout_user(
) -> None:
    """Log user out of system."""
    if not BaseUserApi.subclasses:
        raise HTTPException(status_code=500, detail="Not implemented")
    return await BaseUserApi.subclasses[0]().logout_user()


@router.get(
    "/user/{username}",
    responses={
        200: {"model": User, "description": "successful operation"},
        400: {"description": "Invalid username supplied"},
        404: {"description": "User not found"},
        "default": {"model": Error, "description": "Unexpected error"},
    },
    tags=["user"],
    summary="Get user by user name.",
    response_model_by_alias=True,
)
async def get_user_by_name(
    username: Annotated[StrictStr, Field(description="The name that needs to be fetched. Use user1 for testing")] = Path(..., description="The name that needs to be fetched. Use user1 for testing"),
) -> User:
    """Get user details based on username."""
    if not BaseUserApi.subclasses:
        raise HTTPException(status_code=500, detail="Not implemented")
    return await BaseUserApi.subclasses[0]().get_user_by_name(username)


@router.put(
    "/user/{username}",
    responses={
        200: {"description": "successful operation"},
        "default": {"model": Error, "description": "Unexpected error"},
    },
    tags=["user"],
    summary="Update user.",
    response_model_by_alias=True,
)
async def update_user(
    username: Annotated[StrictStr, Field(description="name that need to be deleted")] = Path(..., description="name that need to be deleted"),
    user: Annotated[Optional[User], Field(description="Update an existent user in the store")] = Body(None, description="Update an existent user in the store"),
) -> None:
    """This can only be done by the logged in user."""
    if not BaseUserApi.subclasses:
        raise HTTPException(status_code=500, detail="Not implemented")
    return await BaseUserApi.subclasses[0]().update_user(username, user)


@router.delete(
    "/user/{username}",
    responses={
        200: {"description": "successful operation"},
        400: {"description": "Invalid username supplied"},
        404: {"description": "User not found"},
        "default": {"model": Error, "description": "Unexpected error"},
    },
    tags=["user"],
    summary="Delete user.",
    response_model_by_alias=True,
)
async def delete_user(
    username: Annotated[StrictStr, Field(description="The name that needs to be deleted")] = Path(..., description="The name that needs to be deleted"),
) -> None:
    """This can only be done by the logged in user."""
    if not BaseUserApi.subclasses:
        raise HTTPException(status_code=500, detail="Not implemented")
    return await BaseUserApi.subclasses[0]().delete_user(username)
