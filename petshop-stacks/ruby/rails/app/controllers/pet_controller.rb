class PetController < ApplicationController
  def create
    pet = parsed_body
    result = repo.add_pet(pet)
    render json: result
  rescue => e
    render json: { message: e.message }, status: :bad_request
  end

  def update_pet
    pet = parsed_body
    result = repo.update_pet(pet)
    render json: result
  rescue NotFoundError => e
    render json: { message: e.message }, status: :not_found
  rescue => e
    render json: { message: e.message }, status: :bad_request
  end

  def find_pets_by_status
    status = params[:status] || 'available'
    result = repo.find_pets_by_status(status)
    render json: result
  rescue => e
    render json: { message: e.message }, status: :bad_request
  end

  def find_pets_by_tags
    tags = Array(params[:tags])
    result = repo.find_pets_by_tags(tags)
    render json: result
  rescue => e
    render json: { message: e.message }, status: :bad_request
  end

  def show
    pet = repo.find_pet_by_id(params[:petId])
    render json: pet
  rescue NotFoundError => e
    render json: { message: e.message }, status: :not_found
  end

  def destroy
    repo.delete_pet(params[:petId])
    render json: {}
  rescue => e
    render json: { message: e.message }, status: :bad_request
  end

  def update_pet_with_form
    repo.update_pet_with_form(
      params[:petId],
      params[:name],
      params[:status]
    )
    render json: {}
  rescue NotFoundError => e
    render json: { message: e.message }, status: :not_found
  rescue => e
    render json: { message: e.message }, status: :bad_request
  end

  def upload_file
    content = request.body.read
    byte_count = repo.upload_file(
      params[:petId],
      content,
      request.content_type || 'application/octet-stream',
      params[:additionalMetadata]
    )
    render json: {
      code: 200,
      type: 'application/octet-stream',
      message: "File uploaded for pet #{params[:petId]}, #{byte_count} bytes"
    }
  rescue NotFoundError => e
    render json: { message: e.message }, status: :not_found
  rescue => e
    render json: { message: e.message }, status: :bad_request
  end
end
