# coding: utf-8

from typing import Dict, List  # noqa: F401
import importlib
import pkgutil

from petstore.apis.store_api_base import BaseStoreApi
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
from pydantic import Field, StrictInt
from typing import Any, Dict, Optional
from typing_extensions import Annotated
from petstore.models.error import Error
from petstore.models.order import Order
from petstore.security_api import get_token_api_key

router = APIRouter()

ns_pkg = petstore.petstore.impl
for _, name, _ in pkgutil.iter_modules(ns_pkg.__path__, ns_pkg.__name__ + "."):
    importlib.import_module(name)


@router.get(
    "/store/inventory",
    responses={
        200: {"model": Dict[str, int], "description": "successful operation"},
        "default": {"model": Error, "description": "Unexpected error"},
    },
    tags=["store"],
    summary="Returns pet inventories by status.",
    response_model_by_alias=True,
)
async def get_inventory(
    token_api_key: TokenModel = Security(
        get_token_api_key
    ),
) -> Dict[str, int]:
    """Returns a map of status codes to quantities."""
    if not BaseStoreApi.subclasses:
        raise HTTPException(status_code=500, detail="Not implemented")
    return await BaseStoreApi.subclasses[0]().get_inventory()


@router.post(
    "/store/order",
    responses={
        200: {"model": Order, "description": "successful operation"},
        400: {"description": "Invalid input"},
        422: {"description": "Validation exception"},
        "default": {"model": Error, "description": "Unexpected error"},
    },
    tags=["store"],
    summary="Place an order for a pet.",
    response_model_by_alias=True,
)
async def place_order(
    order: Optional[Order] = Body(None, description=""),
) -> Order:
    """Place a new order in the store."""
    if not BaseStoreApi.subclasses:
        raise HTTPException(status_code=500, detail="Not implemented")
    return await BaseStoreApi.subclasses[0]().place_order(order)


@router.get(
    "/store/order/{orderId}",
    responses={
        200: {"model": Order, "description": "successful operation"},
        400: {"description": "Invalid ID supplied"},
        404: {"description": "Order not found"},
        "default": {"model": Error, "description": "Unexpected error"},
    },
    tags=["store"],
    summary="Find purchase order by identifier.",
    response_model_by_alias=True,
)
async def get_order_by_id(
    orderId: Annotated[StrictInt, Field(description="ID of order that needs to be fetched")] = Path(..., description="ID of order that needs to be fetched"),
) -> Order:
    """For valid response try integer IDs with value &lt;&#x3D; 5 or &gt; 10. Other values will generate exceptions."""
    if not BaseStoreApi.subclasses:
        raise HTTPException(status_code=500, detail="Not implemented")
    return await BaseStoreApi.subclasses[0]().get_order_by_id(orderId)


@router.delete(
    "/store/order/{orderId}",
    responses={
        200: {"description": "successful operation"},
        400: {"description": "Invalid ID supplied"},
        404: {"description": "Order not found"},
        "default": {"model": Error, "description": "Unexpected error"},
    },
    tags=["store"],
    summary="Delete purchase order by identifier.",
    response_model_by_alias=True,
)
async def delete_order(
    orderId: Annotated[StrictInt, Field(description="ID of the order that needs to be deleted")] = Path(..., description="ID of the order that needs to be deleted"),
) -> None:
    """For valid response try integer IDs with value &lt; 1000. Anything above 1000 or non-integers will generate API errors."""
    if not BaseStoreApi.subclasses:
        raise HTTPException(status_code=500, detail="Not implemented")
    return await BaseStoreApi.subclasses[0]().delete_order(orderId)
