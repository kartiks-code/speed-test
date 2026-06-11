class StoreController < ApplicationController
  def get_inventory
    result = repo.get_inventory
    render json: result
  rescue => e
    render json: { message: e.message }, status: :internal_server_error
  end

  def place_order
    order = parsed_body
    result = repo.place_order(order)
    render json: result, status: :created
  rescue => e
    render json: { message: e.message }, status: :bad_request
  end

  def get_order_by_id
    order = repo.find_order_by_id(params[:orderId])
    render json: order
  rescue NotFoundError => e
    render json: { message: e.message }, status: :not_found
  end

  def delete_order
    repo.delete_order(params[:orderId])
    render json: {}
  rescue => e
    render json: { message: e.message }, status: :bad_request
  end
end
