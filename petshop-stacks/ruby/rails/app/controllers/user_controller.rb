class UserController < ApplicationController
  def create
    user = parsed_body
    repo.create_user(user)
    render json: {}
  rescue => e
    render json: { message: e.message }, status: :bad_request
  end

  def create_users_with_list_input
    users = parsed_body
    users = [users] unless users.is_a?(Array)
    repo.create_users_with_list(users)
    render json: {}
  rescue => e
    render json: { message: e.message }, status: :bad_request
  end

  def show
    user = repo.find_user_by_username(params[:username])
    render json: user
  rescue NotFoundError => e
    render json: { message: e.message }, status: :not_found
  end

  def update
    user = parsed_body
    repo.update_user(params[:username], user)
    render json: {}
  rescue NotFoundError => e
    render json: { message: e.message }, status: :not_found
  rescue => e
    render json: { message: e.message }, status: :bad_request
  end

  def destroy
    repo.delete_user(params[:username])
    render json: {}
  rescue => e
    render json: { message: e.message }, status: :bad_request
  end

  def login_user
    username = params[:username]
    password = params[:password]
    success = repo.login_user(username, password)
    if success
      render json: { message: "logged in user session: #{username}" }
    else
      render json: { message: 'Invalid username/password supplied' }, status: :bad_request
    end
  rescue => e
    render json: { message: e.message }, status: :bad_request
  end

  def logout_user
    render json: { message: 'ok' }
  end
end
