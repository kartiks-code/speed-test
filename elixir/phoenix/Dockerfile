FROM elixir:latest

WORKDIR /app

RUN mix local.hex --force && mix local.rebar --force

COPY mix.exs mix.lock ./

RUN mix deps.get

COPY . .

RUN mix compile

EXPOSE 8080

ENV PHX_SERVER=true

CMD ["mix", "phx.server"]
