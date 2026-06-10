# coding: utf-8

from typing import ClassVar, Dict, List, Tuple  # noqa: F401

from pydantic import Field, StrictInt
from typing import Any, Dict, Optional
from typing_extensions import Annotated
from petstore.models.error import Error
from petstore.models.order import Order
from petstore.security_api import get_token_api_key

class BaseStoreApi:
    subclasses: ClassVar[Tuple] = ()

    def __init_subclass__(cls, **kwargs):
        super().__init_subclass__(**kwargs)
        BaseStoreApi.subclasses = BaseStoreApi.subclasses + (cls,)
    async def get_inventory(
        self,
    ) -> Dict[str, int]:
        """Returns a map of status codes to quantities."""
        ...


    async def place_order(
        self,
        order: Optional[Order],
    ) -> Order:
        """Place a new order in the store."""
        ...


    async def get_order_by_id(
        self,
        orderId: Annotated[StrictInt, Field(description="ID of order that needs to be fetched")],
    ) -> Order:
        """For valid response try integer IDs with value &lt;&#x3D; 5 or &gt; 10. Other values will generate exceptions."""
        ...


    async def delete_order(
        self,
        orderId: Annotated[StrictInt, Field(description="ID of the order that needs to be deleted")],
    ) -> None:
        """For valid response try integer IDs with value &lt; 1000. Anything above 1000 or non-integers will generate API errors."""
        ...
