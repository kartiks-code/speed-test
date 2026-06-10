# store_api

All URIs are relative to *https://petstore31.swagger.io/api/v3*

Method | HTTP request | Description
------------- | ------------- | -------------
**getInventory**](store_api.md#getInventory) | **GET** /store/inventory | Returns pet inventories by status.
**placeOrder**](store_api.md#placeOrder) | **POST** /store/order | Place an order for a pet.
**deleteOrder**](store_api.md#deleteOrder) | **DELETE** /store/order/{orderId} | Delete purchase order by identifier.
**getOrderById**](store_api.md#getOrderById) | **GET** /store/order/{orderId} | Find purchase order by identifier.


# **getInventory**
> std::collections::HashMap<String, i32> getInventory(ctx, )
Returns pet inventories by status.

Returns a map of status codes to quantities.

### Required Parameters
This endpoint does not need any parameter.

### Return type

[**std::collections::HashMap<String, i32>**](integer.md)

### Authorization

[api_key](../README.md#api_key)

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **placeOrder**
> models::Order placeOrder(optional)
Place an order for a pet.

Place a new order in the store.

### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **optional** | **map[string]interface{}** | optional parameters | nil if no parameters

### Optional Parameters
Optional parameters are passed through a map[string]interface{}.

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **order** | [**Order**](Order.md)|  | 

### Return type

[**models::Order**](Order.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json, application/xml, application/x-www-form-urlencoded
 - **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **deleteOrder**
> deleteOrder(order_id)
Delete purchase order by identifier.

For valid response try integer IDs with value < 1000. Anything above 1000 or non-integers will generate API errors.

### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
  **order_id** | **i64**| ID of the order that needs to be deleted | 

### Return type

 (empty response body)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **getOrderById**
> models::Order getOrderById(order_id)
Find purchase order by identifier.

For valid response try integer IDs with value <= 5 or > 10. Other values will generate exceptions.

### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
  **order_id** | **i64**| ID of order that needs to be fetched | 

### Return type

[**models::Order**](Order.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json, application/xml

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

