# coding: utf-8

from typing import ClassVar, Dict, List, Tuple  # noqa: F401

from pydantic import Field, StrictBytes, StrictInt, StrictStr, field_validator
from typing import Any, List, Optional, Tuple, Union
from typing_extensions import Annotated
from petstore.models.api_response import ApiResponse
from petstore.models.error import Error
from petstore.models.pet import Pet
from petstore.security_api import get_token_petstore_auth, get_token_api_key

class BasePetApi:
    subclasses: ClassVar[Tuple] = ()

    def __init_subclass__(cls, **kwargs):
        super().__init_subclass__(**kwargs)
        BasePetApi.subclasses = BasePetApi.subclasses + (cls,)
    async def update_pet(
        self,
        pet: Annotated[Pet, Field(description="Update an existent pet in the store")],
    ) -> Pet:
        """Update an existing pet by Id."""
        ...


    async def add_pet(
        self,
        pet: Annotated[Pet, Field(description="Create a new pet in the store")],
    ) -> Pet:
        """Add a new pet to the store."""
        ...


    async def find_pets_by_status(
        self,
        status: Annotated[Optional[StrictStr], Field(description="Status values that need to be considered for filter")],
    ) -> List[Pet]:
        """Multiple status values can be provided with comma separated strings."""
        ...


    async def find_pets_by_tags(
        self,
        tags: Annotated[Optional[List[StrictStr]], Field(description="Tags to filter by")],
    ) -> List[Pet]:
        """Multiple tags can be provided with comma separated strings. Use tag1, tag2, tag3 for testing."""
        ...


    async def get_pet_by_id(
        self,
        petId: Annotated[StrictInt, Field(description="ID of pet to return")],
    ) -> Pet:
        """Returns a single pet."""
        ...


    async def update_pet_with_form(
        self,
        petId: Annotated[StrictInt, Field(description="ID of pet that needs to be updated")],
        name: Annotated[Optional[StrictStr], Field(description="Name of pet that needs to be updated")],
        status: Annotated[Optional[StrictStr], Field(description="Status of pet that needs to be updated")],
    ) -> None:
        """update a pet via the form data."""
        ...


    async def delete_pet(
        self,
        petId: Annotated[StrictInt, Field(description="Pet id to delete")],
        api_key: Optional[StrictStr],
    ) -> None:
        """delete a pet."""
        ...


    async def upload_file(
        self,
        petId: Annotated[StrictInt, Field(description="ID of pet to update")],
        additional_metadata: Annotated[Optional[StrictStr], Field(description="Additional Metadata")],
        body: Optional[Union[StrictBytes, StrictStr, Tuple[StrictStr, StrictBytes]]],
    ) -> ApiResponse:
        """Upload an image of pet."""
        ...
