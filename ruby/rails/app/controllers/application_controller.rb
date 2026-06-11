class ApplicationController < ActionController::API
  def repo
    PETSTORE_REPOSITORY
  end

  def parsed_body
    raw = request.body.read
    raw.empty? ? {} : JSON.parse(raw)
  rescue JSON::ParserError
    {}
  end
end
